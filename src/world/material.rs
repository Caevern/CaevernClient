#[derive(Clone)]
pub struct Material {
    pub texture: String,
    pub displacement: String,
    pub color: (f32, f32, f32),
    pub image: Option<image::DynamicImage>,
}
impl Material {
    pub fn from_texture(texture: &str) -> Self {
        Self {
            texture: texture.to_string(),
            displacement: "".to_string(),
            color: (1.0, 1.0, 1.0),
            image: None,
        }
    }

    pub fn from_color(color: (f32, f32, f32)) -> Self {
        Self {
            texture: "".to_string(),
            displacement: "".to_string(),
            color,
            image: None,
        }
    }

    pub fn set_image(&mut self, image: Option<image::DynamicImage>) {
        self.image = image;
    }

    pub fn set_displacement(&mut self, texture: &str) {
        self.displacement = texture.to_string();
    }

    pub fn get_texture(&self) -> &str {
        &self.texture
    }
    pub fn get_displacement(&self) -> Option<&str> {
        if self.displacement != "".to_string() {
            Some(&self.displacement)
        } else {
            None
        }
    }
}
