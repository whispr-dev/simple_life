use std::fs::File;
use std::io::Write;
use std::time::{Duration, Instant};
use rand::Rng;
use minifb::{Key, Window, WindowOptions};

struct SimpleLife {
    width: usize,
    height: usize,
    grid: Vec<f32>,
    kernel: Vec<f32>,
    kernel_radius: usize,
    dt: f32,
}

impl SimpleLife {
    // All your existing methods remain unchanged...
    
    fn new(width: usize, height: usize, kernel_radius: usize, dt: f32) -> Self {
        let mut sim = SimpleLife {
            width,
            height,
            grid: vec![0.0; width * height],
            kernel: vec![0.0; (2 * kernel_radius + 1) * (2 * kernel_radius + 1)],
            kernel_radius,
            dt,
        };
        
        sim.init_kernel();
        sim
    }
    
    fn init_kernel(&mut self) {
        let kernel_size = 2 * self.kernel_radius + 1;
        let mut kernel_sum = 0.0;
        
        for y in 0..kernel_size {
            for x in 0..kernel_size {
                let dx = x as f32 - self.kernel_radius as f32;
                let dy = y as f32 - self.kernel_radius as f32;
                let distance = (dx*dx + dy*dy).sqrt();
                
                // Linear falloff from center
                let value = (1.0 - distance / self.kernel_radius as f32).max(0.0);
                self.kernel[y * kernel_size + x] = value;
                kernel_sum += value;
            }
        }
        
        // Normalize kernel
        for k in &mut self.kernel {
            *k /= kernel_sum;
        }
    }
    
    fn growth_function(&self, u: f32) -> f32 {
        // More forgiving growth function with a wider "alive" range
        1.8 * u * (1.0 - u) - 0.2
    }
    
    fn compute_potential(&self) -> Vec<f32> {
        let mut potential = vec![0.0; self.width * self.height];
        let kernel_size = 2 * self.kernel_radius + 1;
        
        for y in 0..self.height {
            for x in 0..self.width {
                let mut sum = 0.0;
                
                for ky in 0..kernel_size {
                    for kx in 0..kernel_size {
                        let gx = (x + kx + self.width - self.kernel_radius) % self.width;
                        let gy = (y + ky + self.height - self.kernel_radius) % self.height;
                        
                        sum += self.grid[gy * self.width + gx] * self.kernel[ky * kernel_size + kx];
                    }
                }
                
                potential[y * self.width + x] = sum;
            }
        }
        
        potential
    }
    
    fn random_init(&mut self, radius: f32, density: f32) {
        // Clear the grid
        for i in &mut self.grid {
            *i = 0.0;
        }
        
        let center_x = self.width / 2;
        let center_y = self.height / 2;
        let max_r = (self.width.min(self.height) as f32 * radius) as usize;
        let mut rng = rand::thread_rng();
        
        // Create a more structured initial pattern
        for y in 0..self.height {
            for x in 0..self.width {
                let dx = x as isize - center_x as isize;
                let dy = y as isize - center_y as isize;
                let dist = ((dx*dx + dy*dy) as f32).sqrt();
                
                if dist < max_r as f32 {
                    let r: f32 = rng.r#gen();
                    
                    // More cells start alive
                    if r < density {
                        // Higher initial values
                        self.grid[y * self.width + x] = r * 0.5 + 0.3;
                    } else if r < density + 0.2 {
                        // Create some medium-valued cells too
                        self.grid[y * self.width + x] = r * 0.3;
                    }
                }
            }
        }
        
        // Add some stable structures (like a simple "block" pattern)
        if self.width > 50 && self.height > 50 {
            // Add a few stable blocks in different locations
            for i in 0..5 {
                let bx = center_x as isize + (i as isize - 2) * 10;
                let by = center_y as isize + (i as isize - 2) * 10;
                
                if bx > 2 && bx < self.width as isize - 2 && 
                   by > 2 && by < self.height as isize - 2 {
                    // Create a 2x2 block with high values
                    for yi in 0..2 {
                        for xi in 0..2 {
                            self.grid[(by as usize + yi) * self.width + (bx as usize + xi)] = 0.9;
                        }
                    }
                }
            }
        }
    }

    fn update(&mut self) {
        let potential = self.compute_potential();
        let mut has_active_cells = false;
        
        for i in 0..self.grid.len() {
            let growth = self.growth_function(potential[i]);
            self.grid[i] += self.dt * growth;
            self.grid[i] = self.grid[i].clamp(0.0, 1.0);
            
            // Check if we have any active cells
            if self.grid[i] > 0.01 {
                has_active_cells = true;
            }
        }
        
        // Print warning if all cells died
        if !has_active_cells {
            println!("WARNING: All cells have died! The simulation might need adjustment.");
        }
    }

    // New function to convert grid values to a blue-scale color buffer for display
    fn create_buffer(&self) -> Vec<u32> {
        let mut buffer = vec![0; self.width * self.height];
        
        for (i, &value) in self.grid.iter().enumerate() {
            // Convert value from 0.0-1.0 to a blue-scale color
            // We'll use a slight gradient from black to blue to make the visualization more interesting
            let blue = (value * 255.0) as u8;
            let green = (value * value * 100.0) as u8; // Slight green component for medium values
            let red = (value * value * value * 50.0) as u8; // Very slight red for high values
            
            // Pack RGB values into a single u32 (0xRRGGBB format)
            buffer[i] = ((red as u32) << 16) | ((green as u32) << 8) | blue as u32;
        }
        
        buffer
    }

    fn save_image(&self, filename: &str) -> std::io::Result<()> {
        let mut file = File::create(filename)?;
        
        // Write PGM header with proper line endings
        writeln!(file, "P5")?;
        writeln!(file, "{} {}", self.width, self.height)?;
        writeln!(file, "255")?;
        
        // Count non-zero pixels for debugging
        let mut non_zero_pixels = 0;
        
        // Write pixel data
        for value in &self.grid {
            let pixel = (*value * 255.0) as u8;
            file.write_all(&[pixel])?;
            
            if pixel > 0 {
                non_zero_pixels += 1;
            }
        }
        
        println!("Saved image with {} non-zero pixels out of {}", 
                non_zero_pixels, self.width * self.height);
        
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create our simulation with slightly larger dimensions for better visualization
    let width = 400;
    let height = 400;
    let mut sim = SimpleLife::new(width, height, 13, 0.05);
    
    // Initialize with random pattern
    sim.random_init(0.3, 0.3);
    
    // Create a window for visualization
    let mut window = Window::new(
        "SimpleLife - Continuous Cellular Automaton",
        width,
        height,
        WindowOptions::default(),
    )?;
    
    // Set a reasonable update rate (30 fps is good for visualization)
    window.limit_update_rate(Some(Duration::from_micros(5555)));
    
    let mut frame_count = 0;
    let mut last_time = Instant::now();
    let mut fps = 0.0;
    
    // Main loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Update the simulation
        sim.update();
        
        // Convert the grid to a displayable buffer
        let buffer = sim.create_buffer();
        
        // Update the window with the new buffer
        window.update_with_buffer(&buffer, width, height)?;
        
        // Calculate FPS every second
        frame_count += 1;
        let current_time = Instant::now();
        let elapsed = current_time.duration_since(last_time);
        
        if elapsed.as_secs() >= 1 {
            fps = frame_count as f64 / elapsed.as_secs_f64();
            window.set_title(&format!("SimpleLife - FPS: {:.1}", fps));
            frame_count = 0;
            last_time = current_time;
            
            // Print active cells count occasionally
            let active_cells = sim.grid.iter().filter(|&&v| v > 0.01).count();
            println!("Active cells: {} ({:.2}% of grid)", 
                     active_cells, 
                     100.0 * active_cells as f32 / (width * height) as f32);
        }
        
        // Save a frame occasionally if desired (every 100 updates)
        if frame_count % 100 == 0 {
            let filename = format!("simplelife_frame_{:04}.pgm", frame_count / 100);
            sim.save_image(&filename)?;
        }
        
        // Allow user interaction
        if window.is_key_pressed(Key::R, minifb::KeyRepeat::No) {
            println!("Reinitializing simulation...");
            sim.random_init(0.3, 0.3);
        }
    }
    
    println!("Simulation ended successfully!");
    Ok(())
}