use std::fs::File;
use std::io::Write;
use rand::Rng;

struct SimpleLife {
    width: usize,
    height: usize,
    grid: Vec<f32>,
    kernel: Vec<f32>,
    kernel_radius: usize,
    dt: f32,
}

impl SimpleLife {
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
        // Original was: 2.0 * u * (1.0 - u) - 0.5
        1.8 * u * (1.0 - u) - 0.2  // This creates a wider band of survival
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

fn main() -> std::io::Result<()> {
    // Create a smaller simulation to reduce computation time
    let mut sim = SimpleLife::new(200, 200, 13, 0.05);  // Reduced dt from 0.1 to 0.05
    
    // Initialize with random pattern
    sim.random_init(0.3, 0.3);
    
    // Run for 500 steps, saving every 20th frame
    for i in 0..500 {
        sim.update();
    
        if i % 20 == 0 {
            let filename = format!("simplelife_{:03}.pgm", i/20);
            sim.save_image(&filename)?;
            println!("Saved frame {}", i/20);
        }
    }
    
    println!("Simulation completed successfully!");
    Ok(())
}