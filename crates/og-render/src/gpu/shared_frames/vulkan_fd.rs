//! Linux: the slots as Vulkan images whose memory is exported as opaque
//! fds (through wgpu-hal), passed with `SCM_RIGHTS` with the `Hello`.
//!
//! Both sides create the image from [`image_info`], so the create
//! parameters an opaque-fd import requires to match are one function.
//! wgpu-hal enables `VK_KHR_external_memory_fd` where the device has it.

use super::{MAX_SLOTS, SLOT_FORMAT, slot_desc};
use ash::vk;
use serde::{Deserialize, Serialize};
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd, OwnedFd, RawFd};
use std::os::unix::net::UnixStream;
use wgpu::hal::api::Vulkan;

/// What carries a slot's memory between the processes.
pub type SharedHandle = OwnedFd;

const VK_FORMAT: vk::Format = vk::Format::R8G8B8A8_UNORM;
const VK_VIEW_FORMATS: [vk::Format; 2] = [vk::Format::R8G8B8A8_UNORM, vk::Format::R8G8B8A8_SRGB];

/// Engine → host, once a generation, with one fd per slot attached.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Hello {
    pub width: u32,
    pub height: u32,
    pub slots: u32,
    pub allocation_size: u64,
    pub memory_type_index: u32,
    pub device_uuid: [u8; 16],
    pub driver_uuid: [u8; 16],
    pub adapter: String,
}

/// The identity of the physical device and driver behind `device`: an
/// opaque fd is valid only where both match.
fn device_ids(device: &wgpu::Device) -> Result<([u8; 16], [u8; 16], String), String> {
    let hal = unsafe { device.as_hal::<Vulkan>() }.ok_or("not a Vulkan device")?;
    let instance = hal.shared_instance().raw_instance();
    let mut id = vk::PhysicalDeviceIDProperties::default();
    let mut props = vk::PhysicalDeviceProperties2::default().push_next(&mut id);
    unsafe { instance.get_physical_device_properties2(hal.raw_physical_device(), &mut props) };
    let name = props
        .properties
        .device_name_as_c_str()
        .map(|c| c.to_string_lossy().into_owned())
        .unwrap_or_default();
    Ok((id.device_uuid, id.driver_uuid, name))
}

fn image_info<'a>(
    width: u32,
    height: u32,
    external: &'a mut vk::ExternalMemoryImageCreateInfo<'a>,
    formats: &'a mut vk::ImageFormatListCreateInfo<'a>,
) -> vk::ImageCreateInfo<'a> {
    vk::ImageCreateInfo::default()
        .flags(vk::ImageCreateFlags::MUTABLE_FORMAT)
        .image_type(vk::ImageType::TYPE_2D)
        .format(VK_FORMAT)
        .extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(
            vk::ImageUsageFlags::COLOR_ATTACHMENT
                | vk::ImageUsageFlags::SAMPLED
                | vk::ImageUsageFlags::TRANSFER_SRC
                | vk::ImageUsageFlags::TRANSFER_DST,
        )
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .push_next(external)
        .push_next(formats)
}

fn hal_desc(width: u32, height: u32) -> wgpu::hal::TextureDescriptor<'static> {
    wgpu::hal::TextureDescriptor {
        label: Some("shared frame"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: SLOT_FORMAT,
        usage: wgpu::TextureUses::COLOR_TARGET
            | wgpu::TextureUses::RESOURCE
            | wgpu::TextureUses::COPY_SRC
            | wgpu::TextureUses::COPY_DST,
        memory_flags: wgpu::hal::MemoryFlags::empty(),
        view_formats: vec![super::OUTPUT_FORMAT],
    }
}

fn device_local_type(
    instance: &ash::Instance,
    physical: vk::PhysicalDevice,
    bits: u32,
) -> Option<u32> {
    let props = unsafe { instance.get_physical_device_memory_properties(physical) };
    (0..props.memory_type_count).find(|&i| {
        bits & (1 << i) != 0
            && props.memory_types[i as usize]
                .property_flags
                .contains(vk::MemoryPropertyFlags::DEVICE_LOCAL)
    })
}

/// Engine: `slots` exportable images on `device`, their memory as fds and
/// the generation's `Hello`.
pub fn export_slots(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    slots: u32,
) -> Result<(Hello, Vec<wgpu::Texture>, Vec<OwnedFd>), String> {
    if !(1..=MAX_SLOTS).contains(&slots) {
        return Err(format!("1 to {MAX_SLOTS} slots, not {slots}"));
    }
    let (device_uuid, driver_uuid, adapter) = device_ids(device)?;
    let mut textures = Vec::new();
    let mut fds = Vec::new();
    let mut allocation = None;
    for _ in 0..slots {
        let hal = unsafe { device.as_hal::<Vulkan>() }.ok_or("not a Vulkan device")?;
        let raw = hal.raw_device();
        let instance = hal.shared_instance().raw_instance();
        let fd_fn = ash::khr::external_memory_fd::Device::new(instance, raw);

        let mut external = vk::ExternalMemoryImageCreateInfo::default()
            .handle_types(vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD);
        let mut formats = vk::ImageFormatListCreateInfo::default().view_formats(&VK_VIEW_FORMATS);
        let info = image_info(width, height, &mut external, &mut formats);
        let image =
            unsafe { raw.create_image(&info, None) }.map_err(|e| format!("vkCreateImage: {e}"))?;
        let req = unsafe { raw.get_image_memory_requirements(image) };
        let Some(type_index) =
            device_local_type(instance, hal.raw_physical_device(), req.memory_type_bits)
        else {
            unsafe { raw.destroy_image(image, None) };
            return Err("no device-local memory type".into());
        };
        let mut dedicated = vk::MemoryDedicatedAllocateInfo::default().image(image);
        let mut export = vk::ExportMemoryAllocateInfo::default()
            .handle_types(vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD);
        let alloc = vk::MemoryAllocateInfo::default()
            .allocation_size(req.size)
            .memory_type_index(type_index)
            .push_next(&mut dedicated)
            .push_next(&mut export);
        let memory = match unsafe { raw.allocate_memory(&alloc, None) } {
            Ok(m) => m,
            Err(e) => {
                unsafe { raw.destroy_image(image, None) };
                return Err(format!("vkAllocateMemory (export): {e}"));
            }
        };
        let bound = unsafe { raw.bind_image_memory(image, memory, 0) }
            .map_err(|e| format!("vkBindImageMemory: {e}"))
            .and_then(|()| {
                unsafe {
                    fd_fn.get_memory_fd(
                        &vk::MemoryGetFdInfoKHR::default()
                            .memory(memory)
                            .handle_type(vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD),
                    )
                }
                .map_err(|e| format!("vkGetMemoryFdKHR: {e}"))
            });
        let fd = match bound {
            Ok(fd) => fd,
            Err(e) => {
                unsafe {
                    raw.destroy_image(image, None);
                    raw.free_memory(memory, None);
                }
                return Err(e);
            }
        };
        fds.push(unsafe { OwnedFd::from_raw_fd(fd) });
        allocation = Some((req.size, type_index));
        let hal_texture = unsafe {
            hal.texture_from_raw(
                image,
                &hal_desc(width, height),
                None,
                wgpu::hal::vulkan::TextureMemory::Dedicated(memory),
            )
        };
        drop(hal);
        textures.push(unsafe {
            device.create_texture_from_hal::<Vulkan>(
                hal_texture,
                &slot_desc(width, height),
                wgpu::TextureUses::UNINITIALIZED,
            )
        });
    }
    let (allocation_size, memory_type_index) = allocation.ok_or("no slots")?;
    Ok((
        Hello {
            width,
            height,
            slots,
            allocation_size,
            memory_type_index,
            device_uuid,
            driver_uuid,
            adapter,
        },
        textures,
        fds,
    ))
}

/// Host: the memory behind `fd` as a sampled texture on `device` (an
/// initial state of RESOURCE: no layout transition discards what the
/// engine wrote).
pub fn import_slot(
    device: &wgpu::Device,
    hello: &Hello,
    fd: &OwnedFd,
) -> Result<wgpu::Texture, String> {
    let (here_device, here_driver, _) = device_ids(device)?;
    if here_device != hello.device_uuid || here_driver != hello.driver_uuid {
        return Err("the frame's memory comes from another GPU or driver".into());
    }
    let (width, height) = (hello.width, hello.height);
    let hal = unsafe { device.as_hal::<Vulkan>() }.ok_or("not a Vulkan device")?;
    let raw = hal.raw_device();
    let mut external = vk::ExternalMemoryImageCreateInfo::default()
        .handle_types(vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD);
    let mut formats = vk::ImageFormatListCreateInfo::default().view_formats(&VK_VIEW_FORMATS);
    let info = image_info(width, height, &mut external, &mut formats);
    let image =
        unsafe { raw.create_image(&info, None) }.map_err(|e| format!("vkCreateImage: {e}"))?;
    let req = unsafe { raw.get_image_memory_requirements(image) };
    if req.size != hello.allocation_size
        || req.memory_type_bits & (1 << hello.memory_type_index) == 0
    {
        unsafe { raw.destroy_image(image, None) };
        return Err(format!(
            "import does not match the export: size {} vs {}, type bits {:#x} vs index {}",
            req.size, hello.allocation_size, req.memory_type_bits, hello.memory_type_index
        ));
    }
    // Vulkan owns the fd once the allocation succeeds: give it a duplicate.
    let raw_fd: RawFd = fd
        .try_clone()
        .map_err(|e| format!("dup: {e}"))?
        .into_raw_fd();
    let mut dedicated = vk::MemoryDedicatedAllocateInfo::default().image(image);
    let mut import = vk::ImportMemoryFdInfoKHR::default()
        .handle_type(vk::ExternalMemoryHandleTypeFlags::OPAQUE_FD)
        .fd(raw_fd);
    let alloc = vk::MemoryAllocateInfo::default()
        .allocation_size(hello.allocation_size)
        .memory_type_index(hello.memory_type_index)
        .push_next(&mut dedicated)
        .push_next(&mut import);
    let memory = match unsafe { raw.allocate_memory(&alloc, None) } {
        Ok(m) => m,
        Err(e) => {
            drop(unsafe { OwnedFd::from_raw_fd(raw_fd) });
            unsafe { raw.destroy_image(image, None) };
            return Err(format!("vkAllocateMemory (import): {e}"));
        }
    };
    unsafe { raw.bind_image_memory(image, memory, 0) }
        .map_err(|e| format!("vkBindImageMemory: {e}"))?;
    let hal_texture = unsafe {
        hal.texture_from_raw(
            image,
            &hal_desc(width, height),
            None,
            wgpu::hal::vulkan::TextureMemory::Dedicated(memory),
        )
    };
    drop(hal);
    Ok(unsafe {
        device.create_texture_from_hal::<Vulkan>(
            hal_texture,
            &slot_desc(width, height),
            wgpu::TextureUses::RESOURCE,
        )
    })
}

// ── the Hello: length + JSON in one sendmsg, the fds as SCM_RIGHTS ───────

/// Room for `MAX_SLOTS` descriptors in one control message.
#[repr(C)]
union Control {
    header: libc::cmsghdr,
    bytes: [u8; 256],
}

/// Engine: sends `hello` with `fds` attached.
pub fn send_hello(stream: &UnixStream, hello: &Hello, fds: &[OwnedFd]) -> Result<(), String> {
    let body = serde_json::to_vec(hello).map_err(|e| e.to_string())?;
    let mut frame = (body.len() as u32).to_le_bytes().to_vec();
    frame.extend_from_slice(&body);
    let raw: Vec<RawFd> = fds.iter().map(|f| f.as_raw_fd()).collect();
    let fd_bytes = std::mem::size_of_val(raw.as_slice()) as u32;
    let mut control = Control { bytes: [0; 256] };
    let space = unsafe { libc::CMSG_SPACE(fd_bytes) } as usize;
    if space > size_of::<Control>() {
        return Err(format!("{} fds", raw.len()));
    }
    let mut iov = libc::iovec {
        iov_base: frame.as_mut_ptr().cast(),
        iov_len: frame.len(),
    };
    let mut msg: libc::msghdr = unsafe { std::mem::zeroed() };
    msg.msg_iov = &mut iov;
    msg.msg_iovlen = 1;
    msg.msg_control = (&raw mut control).cast();
    msg.msg_controllen = space as _;
    unsafe {
        let cmsg = libc::CMSG_FIRSTHDR(&msg);
        (*cmsg).cmsg_level = libc::SOL_SOCKET;
        (*cmsg).cmsg_type = libc::SCM_RIGHTS;
        (*cmsg).cmsg_len = libc::CMSG_LEN(fd_bytes) as _;
        std::ptr::copy_nonoverlapping(
            raw.as_ptr().cast::<u8>(),
            libc::CMSG_DATA(cmsg),
            fd_bytes as usize,
        );
    }
    let sent = unsafe { libc::sendmsg(stream.as_raw_fd(), &msg, libc::MSG_NOSIGNAL) };
    if sent < 0 {
        return Err(format!("sendmsg: {}", std::io::Error::last_os_error()));
    }
    if sent as usize != frame.len() {
        return Err("short hello".into());
    }
    Ok(())
}

/// Host: the hello on `stream` and its fds, one per slot.
pub fn recv_hello(stream: &UnixStream) -> Result<(Hello, Vec<OwnedFd>), String> {
    let mut buf = vec![0u8; 64 * 1024];
    let mut control = Control { bytes: [0; 256] };
    let mut iov = libc::iovec {
        iov_base: buf.as_mut_ptr().cast(),
        iov_len: buf.len(),
    };
    let mut msg: libc::msghdr = unsafe { std::mem::zeroed() };
    msg.msg_iov = &mut iov;
    msg.msg_iovlen = 1;
    msg.msg_control = (&raw mut control).cast();
    msg.msg_controllen = size_of::<Control>() as _;
    let n = unsafe { libc::recvmsg(stream.as_raw_fd(), &mut msg, libc::MSG_CMSG_CLOEXEC) };
    if n < 0 {
        return Err(format!("recvmsg: {}", std::io::Error::last_os_error()));
    }
    let n = n as usize;
    let mut fds = Vec::new();
    unsafe {
        let mut cmsg = libc::CMSG_FIRSTHDR(&msg);
        while !cmsg.is_null() {
            if (*cmsg).cmsg_level == libc::SOL_SOCKET && (*cmsg).cmsg_type == libc::SCM_RIGHTS {
                let data = libc::CMSG_DATA(cmsg);
                let len = (*cmsg).cmsg_len as usize - (data as usize - cmsg as usize);
                for i in 0..len / size_of::<RawFd>() {
                    let fd = std::ptr::read_unaligned(data.cast::<RawFd>().add(i));
                    fds.push(OwnedFd::from_raw_fd(fd));
                }
            }
            cmsg = libc::CMSG_NXTHDR(&msg, cmsg);
        }
    }
    if n < 4 {
        return Err("short hello".into());
    }
    let len = u32::from_le_bytes(buf[..4].try_into().expect("4 bytes")) as usize;
    if n != 4 + len {
        return Err(format!("hello split across reads ({n} of {})", 4 + len));
    }
    let hello: Hello = serde_json::from_slice(&buf[4..n]).map_err(|e| e.to_string())?;
    if fds.len() != hello.slots as usize {
        return Err(format!("{} fds for {} slots", fds.len(), hello.slots));
    }
    Ok((hello, fds))
}
