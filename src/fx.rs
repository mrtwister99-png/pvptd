// SHADEROVE EFEKTY: zarici strely, kruh dosahu, razove vlny.
#![allow(dead_code)] // derive ShaderType generuje kontrolni funkce, ktere se nevolaji
// Shadery jsou v assets/shaders/*.wgsl, tady jsou k nim materialy a pool razovych vln.
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef, ShaderType};
use bevy::sprite::{AlphaMode2d, Material2d, Material2dPlugin};

#[derive(ShaderType, Clone, Copy, Debug, Default)]
pub struct FxParams {
    pub color: Vec4,
    pub extra: Vec4,
}

fn params(color: Color, extra: Vec4) -> FxParams {
    let c = color.to_linear();
    FxParams { color: Vec4::new(c.red, c.green, c.blue, c.alpha), extra }
}

macro_rules! fx_material {
    ($name:ident, $path:literal) => {
        #[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
        pub struct $name {
            #[uniform(0)]
            pub params: FxParams,
        }
        impl Material2d for $name {
            fn fragment_shader() -> ShaderRef { $path.into() }
            fn alpha_mode(&self) -> AlphaMode2d { AlphaMode2d::Blend }
        }
    };
}
fx_material!(GlowMaterial, "shaders/glow.wgsl");
fx_material!(RangeMaterial, "shaders/range.wgsl");
fx_material!(ShockwaveMaterial, "shaders/shockwave.wgsl");

pub struct FxPlugin;
impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            Material2dPlugin::<GlowMaterial>::default(),
            Material2dPlugin::<RangeMaterial>::default(),
            Material2dPlugin::<ShockwaveMaterial>::default(),
        ))
        .add_plugins(StatusFxPlugin);
    }
}

// sdileny ctverec 1x1 (velikost efektu se dela pres scale) + zarici material pro kazdy druh veze
#[derive(Resource)]
pub struct FxAssets {
    pub quad: Handle<Mesh>,
    pub glow: std::collections::HashMap<u8, Handle<GlowMaterial>>,
}

impl FxAssets {
    // zarici material pro druh veze (vytvori se pri prvni strele toho druhu)
    pub fn glow_for(&mut self, kind: u8, color: Color, mats: &mut Assets<GlowMaterial>) -> Handle<GlowMaterial> {
        self.glow.entry(kind).or_insert_with(|| mats.add(GlowMaterial { params: params(color, Vec4::new(kind as f32 * 1.7, 0.0, 0.0, 0.0)) })).clone()
    }
}

// pool razovych vln: kazda ma vlastni material (zacatek a barva v uniformu), bere se dokola nejstarsi
#[derive(Resource, Default)]
pub struct ShockwavePool { pub items: Vec<(Entity, Handle<ShockwaveMaterial>)>, pub next: usize }

const SHOCKWAVE_POOL_SIZE: usize = 48;

pub fn setup_fx(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut waves: ResMut<Assets<ShockwaveMaterial>>) {
    let quad = meshes.add(Rectangle::new(1.0, 1.0));
    let mut pool = ShockwavePool::default();
    for _ in 0..SHOCKWAVE_POOL_SIZE {
        // zacatek v minulosti = neviditelna (shader ji zahodi)
        let mat = waves.add(ShockwaveMaterial { params: params(Color::WHITE, Vec4::new(-100.0, 0.5, 0.0, 0.0)) });
        let e = commands.spawn((Mesh2d(quad.clone()), MeshMaterial2d(mat.clone()), Transform::from_xyz(0.0, 0.0, 6.5))).id();
        pool.items.push((e, mat));
    }
    commands.insert_resource(pool);
    commands.insert_resource(FxAssets { quad, glow: Default::default() });
}

// spusti razovou vlnu v miste `pos` o polomeru `radius`
pub fn spawn_shockwave(commands: &mut Commands, pool: &mut ShockwavePool, mats: &mut Assets<ShockwaveMaterial>, now: f32, pos: Vec2, radius: f32, color: Color, secs: f32) {
    if pool.items.is_empty() { return; }
    let (e, mat) = pool.items[pool.next].clone();
    pool.next = (pool.next + 1) % pool.items.len();
    if let Some(m) = mats.get_mut(&mat) { m.params = params(color, Vec4::new(now, secs, 0.0, 0.0)); }
    if let Some(mut ec) = commands.get_entity(e) {
        ec.insert(Transform::from_xyz(pos.x, pos.y, 6.5).with_scale(Vec3::new(radius * 2.0, radius * 2.0, 1.0)));
    }
}

// material kruhu dosahu s barvou veze a tloustkou okraje podle polomeru
pub fn range_params(color: Color, radius: f32) -> FxParams {
    params(color, Vec4::new(3.0 / radius.max(1.0), 0.0, 0.0, 0.0))
}


// === ZPOMALENI: shaderovy obrys kolem creepa (assets/shaders/status.wgsl) ===
fx_material!(StatusMaterial, "shaders/status.wgsl");

pub struct StatusFxPlugin;
impl Plugin for StatusFxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<StatusMaterial>::default())
            .add_systems(Update, slow_ring_swap_system);
    }
}

// kazdy novy obrys zpomaleni (SlowOutline) dostane misto plneho kruhu shaderovy kruzek;
// zmensovani podle zbyvajiciho casu dela dal update_outline pres scale
fn slow_ring_swap_system(
    mut commands: Commands,
    q: Query<Entity, Added<crate::components::SlowOutline>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StatusMaterial>>,
    mut cache: Local<Option<(Handle<Mesh>, Handle<StatusMaterial>)>>,
) {
    if q.is_empty() { return; }
    let (quad, mat) = cache.get_or_insert_with(|| (
        meshes.add(Rectangle::new(2.0, 2.0)), // 2x2: scale = polomer (stejne jako puvodni kruh)
        mats.add(StatusMaterial { params: params(Color::srgba(0.35, 0.75, 1.0, 0.95), Vec4::ZERO) }),
    )).clone();
    for e in q.iter() {
        if let Some(mut ec) = commands.get_entity(e) {
            ec.remove::<MeshMaterial2d<ColorMaterial>>().try_insert((Mesh2d(quad.clone()), MeshMaterial2d(mat.clone())));
        }
    }
}
