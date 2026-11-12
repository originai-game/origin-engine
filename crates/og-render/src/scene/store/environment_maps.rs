use super::*;

impl RenderScene {
    /// Adds a prefiltered environment map (an asset; og-cook's `type
    /// environment`); [`RenderScene::environment_map`] picks the one a
    /// frame is lit by. A slot freed by [`RenderScene::remove_environment_map`]
    /// is reused under a new generation.
    pub fn add_environment_map(&mut self, map: crate::EnvironmentMapData) -> EnvironmentMapHandle {
        let (i, generation, reused) = self
            .environment_map_slots
            .take(self.environment_maps.len(), true);
        if reused {
            self.environment_maps[i] = map;
            self.environment_map_revisions[i] += 1;
            self.environment_equirects[i] = None;
        } else {
            self.environment_maps.push(map);
            self.environment_map_revisions.push(0);
            self.environment_equirects.push(None);
        }
        EnvironmentMapHandle::from_parts(i, generation)
    }

    /// Adds an environment map from an equirectangular image the renderer
    /// prefilters itself, on the GPU when it uploads it (run-time loads of
    /// uncooked `.hdr` and `.exr`), with its `irradiance` already gathered
    /// ([`crate::EquirectIrradiance`]). [`RenderScene::environment_map_data`]
    /// answers its size and irradiance with no texels. Refused when the
    /// image or the size is not valid.
    pub fn add_environment_equirect(
        &mut self,
        image: crate::EnvironmentEquirect,
        irradiance: [[f32; 3]; 9],
    ) -> Result<EnvironmentMapHandle, String> {
        image.validate()?;
        if irradiance.iter().flatten().any(|v| !v.is_finite()) {
            return Err("an environment map's irradiance is not finite".into());
        }
        let size = image.size;
        let handle = self.add_environment_map(crate::EnvironmentMapData {
            size,
            texels: Vec::new(),
            irradiance,
            sun: Default::default(),
        });
        self.environment_equirects[handle.index()] = Some(std::sync::Arc::new(image));
        Ok(handle)
    }

    /// Slot `index`'s equirectangular image, when the renderer bakes it.
    pub(crate) fn environment_equirect(&self, index: usize) -> Option<&crate::EnvironmentEquirect> {
        self.environment_equirects.get(index)?.as_deref()
    }

    /// Removes an environment map: the renderer frees its cube before the
    /// next frame and a later add reuses the slot under a new generation.
    /// A frame whose [`RenderScene::environment_map`] still names it is
    /// refused ([`crate::RenderError::InvalidHandle`]): light the scene
    /// with another map or the sky first. False for a handle that is not
    /// live.
    pub fn remove_environment_map(&mut self, handle: EnvironmentMapHandle) -> bool {
        let i = handle.index();
        if !self.environment_map_slots.live(i, handle.generation()) {
            return false;
        }
        self.environment_maps[i] = crate::EnvironmentMapData {
            size: 0,
            texels: Vec::new(),
            irradiance: [[0.0; 3]; 9],
            sun: Default::default(),
        };
        self.environment_map_revisions[i] += 1;
        self.environment_equirects[i] = None;
        self.environment_map_slots.remove(i, true);
        true
    }

    /// Whether `handle` names an environment map of this scene that was
    /// not removed.
    pub fn contains_environment_map(&self, handle: EnvironmentMapHandle) -> bool {
        self.environment_map_slots
            .live(handle.index(), handle.generation())
    }

    /// A live environment map's data.
    pub fn environment_map_data(
        &self,
        handle: EnvironmentMapHandle,
    ) -> Option<&crate::EnvironmentMapData> {
        self.contains_environment_map(handle)
            .then(|| &self.environment_maps[handle.index()])
    }

    pub(crate) fn environment_maps(&self) -> &[crate::EnvironmentMapData] {
        &self.environment_maps
    }

    pub(crate) fn environment_map_revisions(&self) -> &[u64] {
        &self.environment_map_revisions
    }

    /// Whether slot `index` holds a live environment map, and its generation.
    pub(crate) fn environment_map_slot(&self, index: usize) -> (bool, u8) {
        let g = self.environment_map_slots.generation(index);
        (self.environment_map_slots.live(index, g), g)
    }
}
