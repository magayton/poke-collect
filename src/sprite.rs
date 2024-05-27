use serde::Deserialize;

// Struct holding pokemon data for sprites 

#[derive(Deserialize, Debug)]
pub struct Sprites {
    back_default: Option<String>,
    back_female: Option<String>,
    back_shiny: Option<String>,
    back_shiny_female: Option<String>,
    front_default: Option<String>,
    front_female: Option<String>,
    front_shiny: Option<String>,
    front_shiny_female: Option<String>,
    other: Option<Other>,
    versions: Option<Versions>,
}

#[derive(Deserialize, Debug)]
struct Other {
    dream_world: Option<DreamWorld>,
    home: Option<Home>,
    #[serde(rename = "official-artwork")]
    official_artwork: Option<OfficialArtwork>,
    showdown: Option<Showdown>,
}

#[derive(Deserialize, Debug)]
struct DreamWorld {
    front_default: Option<String>,
    front_female: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Home {
    front_default: Option<String>,
    front_female: Option<String>,
    front_shiny: Option<String>,
    front_shiny_female: Option<String>,
}

#[derive(Deserialize, Debug)]
struct OfficialArtwork {
    front_default: Option<String>,
    front_shiny: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Showdown {
    back_default: Option<String>,
    back_female: Option<String>,
    back_shiny: Option<String>,
    back_shiny_female: Option<String>,
    front_default: Option<String>,
    front_female: Option<String>,
    front_shiny: Option<String>,
    front_shiny_female: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Versions {
    #[serde(rename = "generation-i")]
    generation_i: Option<GenerationI>,
    #[serde(rename = "generation-ii")]
    generation_ii: Option<GenerationII>,
    #[serde(rename = "generation-iii")]
    generation_iii: Option<GenerationIII>,
    #[serde(rename = "generation-iv")]
    generation_iv: Option<GenerationIV>,
    #[serde(rename = "generation-v")]
    generation_v: Option<GenerationV>,
    #[serde(rename = "generation-vi")]
    generation_vi: Option<GenerationVI>,
    #[serde(rename = "generation-vii")]
    generation_vii: Option<GenerationVII>,
    #[serde(rename = "generation-viii")]
    generation_viii: Option<GenerationVIII>,
}

#[derive(Deserialize, Debug)]
struct GenerationI {
    #[serde(rename = "red-blue")]
    red_blue: Option<RedBlue>,
    yellow: Option<Yellow>,
}

#[derive(Deserialize, Debug)]
struct RedBlue {
    back_default: Option<String>,
    back_gray: Option<String>,
    back_transparent: Option<String>,
    front_default: Option<String>,
    front_gray: Option<String>,
    front_transparent: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Yellow {
    back_default: Option<String>,
    back_gray: Option<String>,
    back_transparent: Option<String>,
    front_default: Option<String>,
    front_gray: Option<String>,
    front_transparent: Option<String>,
}

#[derive(Deserialize, Debug)]
struct GenerationII {
    crystal: Option<Crystal>,
    gold: Option<Gold>,
    silver: Option<Silver>,
}

#[derive(Deserialize, Debug)]
struct Crystal {
    back_default: Option<String>,
    back_shiny: Option<String>,
    back_shiny_transparent: Option<String>,
    back_transparent: Option<String>,
    front_default: Option<String>,
    front_shiny: Option<String>,
    front_shiny_transparent: Option<String>,
    front_transparent: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Gold {
    back_default: Option<String>,
    back_shiny: Option<String>,
    front_default: Option<String>,
    front_shiny: Option<String>,
    front_transparent: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Silver {
    back_default: Option<String>,
    back_shiny: Option<String>,
    front_default: Option<String>,
    front_shiny: Option<String>,
    front_transparent: Option<String>,
}

#[derive(Deserialize, Debug)]
struct GenerationIII {
    emerald: Option<Emerald>,
    #[serde(rename = "firered-leafgreen")]
    firered_leafgreen: Option<FireRedLeafGreen>,
    #[serde(rename = "ruby-sapphire")]
    ruby_sapphire: Option<RubySapphire>,
}

#[derive(Deserialize, Debug)]
struct Emerald {
    front_default: Option<String>,
    front_shiny: Option<String>,
}

#[derive(Deserialize, Debug)]
struct FireRedLeafGreen {
    back_default: Option<String>,
    back_shiny: Option<String>,
    front_default: Option<String>,
    front_shiny: Option<String>,
}

#[derive(Deserialize, Debug)]
struct RubySapphire {
    back_default: Option<String>,
    back_shiny: Option<String>,
    front_default: Option<String>,
    front_shiny: Option<String>,
}

#[derive(Deserialize, Debug)]
struct GenerationIV {
    #[serde(rename = "diamond-pearl")]
    diamond_pearl: Option<DiamondPearl>,
    #[serde(rename = "heartgold-soulsilver")]
    heartgold_soulsilver: Option<HeartGoldSoulSilver>,
    platinum: Option<Platinum>,
}

#[derive(Deserialize, Debug)]
struct DiamondPearl {
    back_default: Option<String>,
    back_female: Option<String>,
    back_shiny: Option<String>,
    back_shiny_female: Option<String>,
    front_default: Option<String>,
    front_female: Option<String>,
    front_shiny: Option<String>,
    front_shiny_female: Option<String>,
}

#[derive(Deserialize, Debug)]
struct HeartGoldSoulSilver {
    back_default: Option<String>,
    back_female: Option<String>,
    back_shiny: Option<String>,
    back_shiny_female: Option<String>,
    front_default: Option<String>,
    front_female: Option<String>,
    front_shiny: Option<String>,
    front_shiny_female: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Platinum {
    back_default: Option<String>,
    back_female: Option<String>,
    back_shiny: Option<String>,
    back_shiny_female: Option<String>,
    front_default: Option<String>,
    front_female: Option<String>,
    front_shiny: Option<String>,
    front_shiny_female: Option<String>,
}

#[derive(Deserialize, Debug)]
struct GenerationV {
    #[serde(rename = "black-white")]
    black_white: Option<BlackWhite>,
}

#[derive(Deserialize, Debug)]
struct BlackWhite {
    animated: Option<Animated>,
    back_default: Option<String>,
    back_female: Option<String>,
    back_shiny: Option<String>,
    back_shiny_female: Option<String>,
    front_default: Option<String>,
    front_female: Option<String>,
    front_shiny: Option<String>,
    front_shiny_female: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Animated {
    back_default: Option<String>,
    back_female: Option<String>,
    back_shiny: Option<String>,
    back_shiny_female: Option<String>,
    front_default: Option<String>,
    front_female: Option<String>,
    front_shiny: Option<String>,
    front_shiny_female: Option<String>,
}

#[derive(Deserialize, Debug)]
struct GenerationVI {
    #[serde(rename = "omegaruby-alphasapphire")]
    omegaruby_alphasapphire: Option<OmegaRubyAlphaSapphire>,
    #[serde(rename = "x-y")]
    x_y: Option<XY>,
}

#[derive(Deserialize, Debug)]
struct OmegaRubyAlphaSapphire {
    front_default: Option<String>,
    front_female: Option<String>,
    front_shiny: Option<String>,
    front_shiny_female: Option<String>,
}

#[derive(Deserialize, Debug)]
struct XY {
    front_default: Option<String>,
    front_female: Option<String>,
    front_shiny: Option<String>,
    front_shiny_female: Option<String>,
}

#[derive(Deserialize, Debug)]
struct GenerationVII {
    icons: Option<Icons>,
    #[serde(rename = "ultra-sun-ultra-moon")]
    ultra_sun_ultra_moon: Option<UltraSunUltraMoon>,
}

#[derive(Deserialize, Debug)]
struct Icons {
    front_default: Option<String>,
    front_female: Option<String>,
}

#[derive(Deserialize, Debug)]
struct UltraSunUltraMoon {
    front_default: Option<String>,
    front_female: Option<String>,
    front_shiny: Option<String>,
    front_shiny_female: Option<String>,
}

#[derive(Deserialize, Debug)]
struct GenerationVIII {
    icons: Option<Icons>,
}
