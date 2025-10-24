// Configuration du système de contours
#[derive(Clone)]
pub struct ContourConfig {
    // Échelle : combien de pixels de la binary map par hexagone
    pub pixels_per_hex: f32,
    // Paramètres du bruit fractal
    pub noise_amplitude: f32,
    pub noise_frequency: f32,
    pub noise_octaves: usize,
    // Seuil pour déterminer terre/mer (0.5 par défaut)
    pub threshold: f32,
    // Tension des splines (0.0 = linéaire, 1.0 = très courbé)
    pub spline_tension: f32,
}

impl Default for ContourConfig {
    fn default() -> Self {
        Self {
            pixels_per_hex: 0.25, // 1 hex = 1/4 pixel (4 hex par pixel)
            noise_amplitude: 0.3,
            noise_frequency: 2.0,
            noise_octaves: 3,
            threshold: 0.5,
            spline_tension: 0.5,
        }
    }
}