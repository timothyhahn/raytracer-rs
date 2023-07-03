use crate::color::Color;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Vec<Color>>,
}

const MAX_COLOR_VALUE: u32 = 255;
const MAX_LINE_LENGTH: u32 = 70;

impl Canvas {
    pub fn new(width: u32, height: u32) -> Canvas {
        let pixels = vec![vec![Color::new(0.0, 0.0, 0.0); width as usize]; height as usize];
        Canvas {
            width,
            height,
            pixels,
        }
    }

    pub fn write_pixel(&mut self, x: u32, y: u32, color: &Color) {
        if x >= self.width || y >= self.height {
            println!(
                "Ignoring pixel at ({}, {}), as canvas size is ({},{})",
                x, y, self.width, self.height
            );
            return;
        }
        self.pixels[y as usize][x as usize] = *color;
    }

    pub fn pixel_at(&self, x: u32, y: u32) -> Color {
        self.pixels[y as usize][x as usize]
    }

    pub fn to_ppm(&self) -> String {
        // Start with the header
        // lines 1-3 of ppm are:
        // P3
        // width height
        // max_color_value
        let mut ppm = String::new();
        ppm.push_str("P3\n");
        ppm.push_str(&format!("{} {}\n", self.width, self.height));
        ppm.push_str(&format!("{}\n", MAX_COLOR_VALUE));

        for row in self.pixels.iter() {
            let mut line = String::new();
            for pixel in row.iter() {
                let r = convert_color_rgb_value_to_ppm_value(pixel.red);
                let g = convert_color_rgb_value_to_ppm_value(pixel.green);
                let b = convert_color_rgb_value_to_ppm_value(pixel.blue);
                line.push_str(&format!("{} {} {} ", r, g, b));
            }
            line.pop(); // Removes space at end

            // Split line if greater than MAX_LINE_LENGTH
            if line.len() > MAX_LINE_LENGTH as usize {
                let mut split_line = String::new();
                // Doing this by color to prevent splitting a color
                let mut words: Vec<&str> = line.split(' ').collect();
                let mut line_length = 0;
                while !words.is_empty() {
                    let word = words.remove(0);
                    line_length += word.len() + 1;
                    if line_length > MAX_LINE_LENGTH as usize {
                        split_line.pop(); // Remove space at end
                        split_line.push('\n');
                        line_length = word.len() + 1;
                    }
                    split_line.push_str(word);
                    split_line.push(' ');
                }
                split_line.pop(); // Removes space at end
                line = split_line;
            }
            line.push('\n');
            ppm.push_str(&line);
        }
        ppm.push('\n');
        ppm
    }

    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        file.write_all(self.to_ppm().as_bytes())?;
        Ok(())
    }
}

fn convert_color_rgb_value_to_ppm_value(value: f64) -> u32 {
    let ppm_value = (value * 255.0).round() as u32;
    if ppm_value > MAX_COLOR_VALUE {
        MAX_COLOR_VALUE
    } else {
        ppm_value
    }
}

#[cfg(test)]
mod tests {
    use crate::canvas::Canvas;
    use crate::canvas::Color;

    #[test]
    fn creating_a_canvas() {
        let c = Canvas::new(10, 20);
        assert_eq!(c.width, 10);
        assert_eq!(c.height, 20);
        for row in c.pixels.iter() {
            for pixel in row.iter() {
                assert_eq!(*pixel, Color::new(0.0, 0.0, 0.0));
            }
        }
    }

    #[test]
    fn can_write_to_canvas() {
        let mut c = Canvas::new(10, 20);
        let red = Color::new(1.0, 0.0, 0.0);
        c.write_pixel(2, 3, &red);
        assert_eq!(c.pixels[3][2], red);
    }

    #[test]
    fn canvas_ignores_pixel_out_of_bounds() {
        let mut c = Canvas::new(10, 20);
        let red = Color::new(1.0, 0.0, 0.0);
        c.write_pixel(10, 20, &red);
        assert_eq!(c.pixels[19][9], Color::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn write_ppm_header() {
        let c = Canvas::new(5, 3);
        let ppm = c.to_ppm();
        let lines: Vec<&str> = ppm.lines().collect();
        assert_eq!(lines[0], "P3");
        assert_eq!(lines[1], "5 3");
        assert_eq!(lines[2], "255")
    }

    #[test]
    fn write_ppm_pixel_data() {
        let mut c = Canvas::new(5, 3);
        let c1 = Color::new(1.5, 0.0, 0.0);
        let c2 = Color::new(0.0, 0.5, 0.0);
        let c3 = Color::new(-0.5, 0.0, 1.0);
        c.write_pixel(0, 0, &c1);
        c.write_pixel(2, 1, &c2);
        c.write_pixel(4, 2, &c3);
        let ppm = c.to_ppm();
        let lines: Vec<&str> = ppm.lines().collect();
        assert_eq!(lines[3], "255 0 0 0 0 0 0 0 0 0 0 0 0 0 0");
        assert_eq!(lines[4], "0 0 0 0 0 0 0 128 0 0 0 0 0 0 0");
        assert_eq!(lines[5], "0 0 0 0 0 0 0 0 0 0 0 0 0 0 255");
    }

    #[test]
    fn split_long_lines_in_ppm() {
        let mut c = Canvas::new(10, 2);
        let color = Color::new(1.0, 0.8, 0.6);
        for row in c.pixels.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = color;
            }
        }
        let ppm = c.to_ppm();
        let lines: Vec<&str> = ppm.lines().collect();
        assert_eq!(
            lines[3],
            "255 204 153 255 204 153 255 204 153 255 204 153 255 204 153 255 204"
        );
        assert_eq!(
            lines[4],
            "153 255 204 153 255 204 153 255 204 153 255 204 153"
        );
        assert_eq!(
            lines[5],
            "255 204 153 255 204 153 255 204 153 255 204 153 255 204 153 255 204"
        );
        assert_eq!(
            lines[6],
            "153 255 204 153 255 204 153 255 204 153 255 204 153"
        );
    }

    #[test]
    fn ppm_files_end_with_newline() {
        let c = Canvas::new(5, 3);
        let ppm = c.to_ppm();
        assert_eq!(ppm.chars().last(), Some('\n'));
    }
}
