//! Abstract rendering surface and an in-memory raster canvas.

/// RGBA color (the original mod draws ARGB; `Color::rgb` drops the alpha
/// where not needed).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color::rgba(r, g, b, 255)
    }

    /// Transparent black, used by tests to check "nothing drawn here".
    pub fn transparent() -> Self {
        Color::rgba(0, 0, 0, 0)
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::transparent()
    }
}

/// The drawing surface the minimap renderer works against.
pub trait Canvas {
    /// Sets one pixel to `color` if it is inside the canvas.
    fn set_pixel(&mut self, x: i32, y: i32, color: Color);

    /// Fills a rectangle, clamped to the canvas.
    fn fill_rect(&mut self, x: i32, y: i32, width: u32, height: u32, color: Color);

    /// Draws a line between two points.
    fn draw_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: Color);

    /// Draws a filled point (radar dots) at `(x, y)`.
    fn draw_point(&mut self, x: i32, y: i32, color: Color, size: u8);
}

/// In-memory RGBA canvas (also used as the CPU raster target in tests).
#[derive(Debug, Clone)]
pub struct RasterCanvas {
    pub width: i32,
    pub height: i32,
    pub pixels: Vec<Color>,
}

impl RasterCanvas {
    pub fn new(width: u32, height: u32) -> Self {
        RasterCanvas {
            width: width as i32,
            height: height as i32,
            pixels: vec![Color::transparent(); (width * height) as usize],
        }
    }

    /// Pixel at `(x, y)`, if it is inside the canvas.
    pub fn get(&self, x: i32, y: i32) -> Option<Color> {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return None;
        }
        Some(self.pixels[(y * self.width + x) as usize])
    }

    /// Number of pixels with a non-zero alpha, for tests.
    pub fn drawn_pixels(&self) -> usize {
        self.pixels.iter().filter(|p| p.a != 0).count()
    }
}

impl Canvas for RasterCanvas {
    fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return;
        }
        self.pixels[(y * self.width + x) as usize] = color;
    }

    fn fill_rect(&mut self, x: i32, y: i32, width: u32, height: u32, color: Color) {
        if width == 0 || height == 0 {
            return;
        }
        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = (x + width as i32).min(self.width);
        let y1 = (y + height as i32).min(self.height);
        for py in y0..y1 {
            for px in x0..x1 {
                self.set_pixel(px, py, color);
            }
        }
    }

    fn draw_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: Color) {
        // Bresenham with float endpoints rounded to nearest.
        let mut x0 = x0.round() as i32;
        let mut y0 = y0.round() as i32;
        let x1 = x1.round() as i32;
        let y1 = y1.round() as i32;
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.set_pixel(x0, y0, color);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    fn draw_point(&mut self, x: i32, y: i32, color: Color, size: u8) {
        let size = u32::from(size.max(1));
        self.fill_rect(x, y, size, size, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raster_canvas_set_and_get() {
        let mut canvas = RasterCanvas::new(10, 10);
        canvas.set_pixel(2, 3, Color::rgb(1, 2, 3));
        assert_eq!(canvas.get(2, 3), Some(Color::rgb(1, 2, 3)));
        assert_eq!(canvas.get(3, 3), Some(Color::transparent()));
        assert_eq!(canvas.get(10, 3), None); // out of bounds (x >= width)
        assert_eq!(canvas.drawn_pixels(), 1);
    }

    #[test]
    fn fill_rect_clamps_to_canvas() {
        let mut canvas = RasterCanvas::new(4, 4);
        canvas.fill_rect(2, 2, 10, 10, Color::rgb(9, 9, 9));
        assert_eq!(canvas.drawn_pixels(), 4);
    }

    #[test]
    fn draw_line_connects_endpoints() {
        let mut canvas = RasterCanvas::new(10, 10);
        canvas.draw_line(0.0, 0.0, 9.0, 9.0, Color::rgb(5, 5, 5));
        assert!(canvas.drawn_pixels() > 0);
        assert_eq!(canvas.get(0, 0), Some(Color::rgb(5, 5, 5)));
    }
}
