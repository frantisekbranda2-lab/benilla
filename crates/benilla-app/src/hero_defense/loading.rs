//! Hero Defense: vlastni nacitaci obrazovka. Obrazek `<HD_ASSETS>/ui/loading.png` (v repu core
//! `core/assets/ui/loading.png`) se kresli pres puvodni loading screen ([`crate::loading_screen`])
//! vzdy, kdyz ten ukazuje obrazek mapy a ukazatel prubehu (vstup do sveta, teleport na jinou
//! mapu). Prepravy (mapa trasy) a cerny prechod zustavaji puvodni.
//!
//! Obrazek vyplni okno bez deformace (presah se orizne). V obrazku je namalovany ukazatel
//! prubehu: pres jeho vnitrek ([`BAR`], v pixelech obrazku) se polozi tmavy zlab a modra vypln
//! podle skutecneho prubehu. Kdyz se obrazek nenacte, zustane puvodni loading screen.
//!
//! Bevy v klientu PNG nenacita (bez funkce `png`), obrazek dekoduje knihovna `image`.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// Vnitrek namalovaneho ukazatele prubehu v pixelech obrazku: vlevo, nahore, vpravo, dole.
const BAR: [f32; 4] = [344.0, 773.0, 1191.0, 806.0];

#[derive(Resource)]
struct HdLoading {
    size: Vec2,
}

#[derive(Component)]
struct HdLoadingRoot;

#[derive(Component)]
struct HdLoadingImage;

#[derive(Component)]
struct HdLoadingTrough;

#[derive(Component)]
struct HdLoadingFill;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup)
        .add_systems(Update, update.after(crate::loading_screen::fit_loading_area));
}

fn assets_dir() -> std::path::PathBuf {
    std::env::var_os("HD_ASSETS")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::current_exe().ok().and_then(|e| e.parent().map(|p| p.join("hd_assets"))))
        .unwrap_or_else(|| "hd_assets".into())
}

fn load_png(path: &std::path::Path) -> Option<Image> {
    let bytes = std::fs::read(path).ok()?;
    let img = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png).ok()?.to_rgba8();
    let (w, h) = img.dimensions();
    Some(Image::new(
        Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        TextureDimension::D2,
        img.into_raw(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    ))
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let path = assets_dir().join("ui").join("loading.png");
    let Some(image) = load_png(&path) else {
        info!("hero defense: nacitaci obrazovka {} nenalezena, zustava puvodni", path.display());
        return;
    };
    let size = Vec2::new(image.width() as f32, image.height() as f32);
    let handle = images.add(image);
    commands.insert_resource(HdLoading { size });
    let abs = || Node { position_type: PositionType::Absolute, ..default() };
    commands
        .spawn((
            HdLoadingRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::BLACK),
            // Nad puvodni nacitaci obrazovkou (1000), pod prechodem obrazovky (1050).
            GlobalZIndex(1001),
            Visibility::Hidden,
        ))
        .with_children(|root| {
            root.spawn((HdLoadingImage, abs(), ImageNode { image: handle, image_mode: NodeImageMode::Stretch, ..default() }));
            root.spawn((HdLoadingTrough, abs(), BackgroundColor(Color::srgb(0.02, 0.03, 0.06))));
            root.spawn((
                HdLoadingFill,
                abs(),
                BackgroundGradient::from(LinearGradient::to_bottom(vec![
                    ColorStop::percent(Color::srgb(0.35, 0.65, 1.0), 0.0),
                    ColorStop::percent(Color::srgb(0.1, 0.35, 0.95), 55.0),
                    ColorStop::percent(Color::srgb(0.05, 0.2, 0.7), 100.0),
                ])),
                BoxShadow::new(Color::srgba(0.3, 0.6, 1.0, 0.6), Val::Px(0.0), Val::Px(0.0), Val::Px(0.0), Val::Px(8.0)),
            ));
        });
}

#[allow(clippy::type_complexity)]
fn update(
    hd: Option<Res<HdLoading>>,
    screen: Option<Res<crate::loading_screen::LoadingScreen>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut root: Query<&mut Visibility, With<HdLoadingRoot>>,
    mut nodes: Query<
        (&mut Node, Has<HdLoadingImage>, Has<HdLoadingFill>),
        Or<(With<HdLoadingImage>, With<HdLoadingTrough>, With<HdLoadingFill>)>,
    >,
) {
    let (Some(hd), Some(screen), Ok(mut vis)) = (hd, screen, root.single_mut()) else {
        return;
    };
    let Some(progress) = screen.hd_overlay() else {
        if *vis != Visibility::Hidden {
            *vis = Visibility::Hidden;
        }
        return;
    };
    if *vis != Visibility::Visible {
        *vis = Visibility::Visible;
    }
    let Some(window) = windows.iter().next() else {
        return;
    };
    // Vyplnit okno (cover): vetsi z obou meritek, presah na stred.
    let (w, h) = (window.width(), window.height());
    let s = (w / hd.size.x).max(h / hd.size.y);
    let (ox, oy) = ((w - hd.size.x * s) / 2.0, (h - hd.size.y * s) / 2.0);
    for (mut node, image, fill) in &mut nodes {
        let (l, t, width, height) = if image {
            (ox, oy, hd.size.x * s, hd.size.y * s)
        } else {
            let full = (BAR[2] - BAR[0]) * s;
            let width = if fill { full * progress.clamp(0.0, 1.0) } else { full };
            (ox + BAR[0] * s, oy + BAR[1] * s, width, (BAR[3] - BAR[1]) * s)
        };
        let (l, t, width, height) = (Val::Px(l), Val::Px(t), Val::Px(width), Val::Px(height));
        if node.left != l || node.top != t || node.width != width || node.height != height {
            node.left = l;
            node.top = t;
            node.width = width;
            node.height = height;
        }
    }
}
