//! Installing the feel systems: their records, slots and the three places they sit in the tick.

use super::*;

/// Installs tweens, flipbooks and 2D skeletons (after the game's systems,
/// which set their clips and IK targets, before og.anim).
pub(crate) fn install_front(world: &mut World, components: &[String]) -> Result<Feel, String> {
    let tweens = og_tween::install(world, components).map_err(err)?;
    let flipbooks = og_anim::install_flipbook(world).map_err(err)?;
    let skeletons = og_anim::install_skeleton2d(world).map_err(err)?;
    Ok(Feel {
        tweens,
        flipbooks,
        skeletons,
        rig_textures: Arc::new(Mutex::new(IdMap::new())),
        tiles: Arc::new(Mutex::new(TileShared::default())),
        shared: Arc::new(Mutex::new(FeelShared::default())),
        sheet_textures: Arc::new(Mutex::new(IdMap::new())),
    })
}

/// The records feel systems read and write, resolved once.
#[derive(Clone)]
pub(crate) struct Records {
    pub transform: (SchemaId, Layout),
    pub collider: (SchemaId, Layout),
    pub tilemap: (SchemaId, Layout),
    pub camera: (SchemaId, Layout),
    pub rail: (SchemaId, Layout),
    pub sprite: (SchemaId, Layout),
    pub flipbook: (SchemaId, Layout),
    pub afterimage: (SchemaId, Layout),
    pub zone: (SchemaId, Layout),
    pub follow: (SchemaId, Layout),
    /// og.VirtualCamera, og.CameraBrain and og.CameraCut (lazy), and
    /// og.PostEffects for a shot's focus.
    pub vcams: vcam::Layouts,
    /// The fields extraction reads of each sprite, flipbook, afterimage
    /// and tile layer row, resolved once against their layouts.
    pub slots: Slots,
}

/// [`Records::slots`].
#[derive(Clone, Copy, Debug)]
pub(crate) struct Slots {
    pub(super) sprite: SpriteSlots,
    pub(super) flipbook: FlipbookSlots,
    pub(super) afterimage: AfterimageSlots,
    pub(super) tilemap: TilemapSlots,
}

impl Records {
    pub(crate) fn of(world: &World, layouts: &BTreeMap<String, Layout>) -> Result<Records, String> {
        let get = |n: &str| -> Result<(SchemaId, Layout), String> {
            let id = world.id_by_name(n).map_err(err)?;
            let layout = match layouts.get(n) {
                Some(l) => l.clone(),
                None => Layout::from_schema(
                    world.schema(id).map_err(err)?,
                    crate::host::codec::RecordKind::Component,
                ),
            };
            Ok((id, layout))
        };
        let (tilemap, sprite) = (get("og.Tilemap")?, get("og.Sprite")?);
        let (flipbook, afterimage) = (get("og.Flipbook")?, get("og.Afterimage")?);
        let slots = Slots {
            sprite: SpriteSlots::of(&sprite.1),
            flipbook: FlipbookSlots::of(&flipbook.1),
            afterimage: AfterimageSlots::of(&afterimage.1),
            tilemap: TilemapSlots::of(&tilemap.1),
        };
        Ok(Records {
            transform: get("og.Transform")?,
            collider: get("og.Collider")?,
            tilemap,
            camera: get("og.Camera")?,
            rail: get("og.CameraRail")?,
            sprite,
            flipbook,
            afterimage,
            zone: get("og.CameraZone")?,
            follow: get("og.Follow2D")?,
            vcams: vcam::Layouts::of(layouts),
            slots,
        })
    }
}

/// Installs og.tilemap and og.camera (after og.anim, before the UI's end and physics).
pub(crate) fn install_middle(
    world: &mut World,
    engine_id: u32,
    feel: &Feel,
    r: &Records,
    physics: &og_physics::PhysicsSystem,
    profile: u32,
) -> Result<(), String> {
    let names =
        |ids: &[&(SchemaId, Layout)]| ids.iter().map(|x| x.1.schema.clone()).collect::<Vec<_>>();
    let tiles = feel.tiles.clone();
    let rec = r.clone();
    let camera_physics = physics.clone();
    let physics = physics.clone();
    world
        .add_system(
            SystemDesc {
                name: TILEMAP_SYSTEM.into(),
                reads: names(&[&r.tilemap]),
                writes: names(&[&r.transform, &r.collider]),
                queries: vec![QueryDesc {
                    read: names(&[&r.tilemap, &r.transform]),
                    ..QueryDesc::default()
                }],
                ..Default::default()
            },
            move |ctx| tilemap_system(ctx, &tiles, &rec, &physics, profile),
        )
        .map_err(err)?;
    world
        .add_state_block(
            TILEMAP_SYSTEM,
            Box::new(TileBlock(feel.tiles.clone(), engine_id)),
        )
        .map_err(err)?;
    let shared = feel.shared.clone();
    let rec = r.clone();
    world
        .add_system(
            SystemDesc {
                name: CAMERA_SYSTEM.into(),
                reads: names(&[&r.rail, &r.transform, &r.zone, &r.follow]),
                writes: names(&[&r.camera]),
                queries: vec![
                    QueryDesc {
                        read: names(&[&r.rail, &r.transform]),
                        ..QueryDesc::default()
                    },
                    QueryDesc {
                        read: names(&[&r.zone]),
                        ..QueryDesc::default()
                    },
                    QueryDesc {
                        read: names(&[&r.follow, &r.transform]),
                        ..QueryDesc::default()
                    },
                ],
                ..Default::default()
            },
            move |ctx| camera_system(ctx, engine_id, &shared, &rec, &camera_physics),
        )
        .map_err(err)?;
    world
        .add_state_block(CAMERA_SYSTEM, Box::new(CameraBlock(feel.shared.clone())))
        .map_err(err)?;
    Ok(())
}

/// Installs og.sprite (after physics and audio: the poses of the tick's end).
pub(crate) fn install_back(world: &mut World, feel: &Feel, r: &Records) -> Result<(), String> {
    let names =
        |ids: &[&(SchemaId, Layout)]| ids.iter().map(|x| x.1.schema.clone()).collect::<Vec<_>>();
    let shared = feel.shared.clone();
    let rec = r.clone();
    world
        .add_system(
            SystemDesc {
                name: SPRITE_SYSTEM.into(),
                reads: names(&[&r.afterimage, &r.sprite, &r.transform, &r.flipbook]),
                writes: Vec::new(),
                queries: vec![QueryDesc {
                    read: names(&[&r.afterimage, &r.sprite, &r.transform]),
                    ..QueryDesc::default()
                }],
                ..Default::default()
            },
            move |ctx| sprite_system(ctx, &shared, &rec),
        )
        .map_err(err)?;
    world
        .add_state_block(SPRITE_SYSTEM, Box::new(SpriteBlock(feel.shared.clone())))
        .map_err(err)?;
    Ok(())
}
