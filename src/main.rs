use pixels::{Pixels, SurfaceTexture};
use rand::Rng;
use std::time::{Duration, Instant};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    // Create an event loop
    let event_loop = EventLoop::new();

    let screen_size = (1000, 1000); // Logical screen size

    // Create a window with logical size
    let window = WindowBuilder::new()
        .with_title("Pixels Demo")
        .with_inner_size(winit::dpi::LogicalSize::new(screen_size.0, screen_size.1))
        .build(&event_loop)
        .unwrap();

    // Get the device pixel ratio (scale factor)
    let scale_factor = window.scale_factor() * 4.0;
    println!("Device Pixel Ratio: {}", scale_factor);

    // Get the actual physical window size (scaled by the device pixel ratio)
    let physical_width = (screen_size.0 as f64 * scale_factor) as u32;
    let physical_height = (screen_size.1 as f64 * scale_factor) as u32;

    // Create a surface texture based on the physical size
    let surface_texture = SurfaceTexture::new(physical_width, physical_height, &window);

    // Create the pixel buffer with the physical size
    let mut pixels =
        Pixels::new(screen_size.0 as u32, screen_size.1 as u32, surface_texture).unwrap();

    // Initialize the frame with random grayscale values
    let mut frame = vec![0u8; (screen_size.0 * screen_size.1 * 4) as usize];
    let mut rng = rand::thread_rng();
    for pixel in frame.chunks_exact_mut(4) {
        let gray_value = rng.gen_range(0..=255) as u8;
        pixel[0] = gray_value; // Red
        pixel[1] = gray_value; // Green
        pixel[2] = gray_value; // Blue
        pixel[3] = 255;        // Alpha
    }

    // Variables for timing
    let mut last_update_inst = Instant::now();
    let start_time = Instant::now(); // Record the start time

    // Start the event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            // Handle window close event
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            // Handle redraw request
            Event::RedrawRequested(_) => {
                let pixels_frame = pixels.frame_mut();
                pixels_frame.copy_from_slice(&frame);

                // Render the frame to the window
                pixels.render().unwrap();
            }
            // Handle main events cleared
            Event::MainEventsCleared => {
                // Update at 60 FPS
                let now = Instant::now();
                let elapsed = now - last_update_inst;
                let total_elapsed = now - start_time; // Calculate total elapsed time

                if total_elapsed >= Duration::from_secs(5) { // Wait for 5 seconds before starting diffusion
                    if elapsed >= Duration::from_secs_f64(1.0 / 60.0) {
                        // Apply the heat equation to diffuse the grayscale values
                        let alpha: f32 = 0.01;  // Thermal diffusivity constant (tunable)
                        let delta_t: f32 = 1.0; // Time step (tunable)
                        let delta_x: f32 = 1.0; // Grid spacing (assuming it's 1.0 for simplicity)

                        let mut new_frame = frame.clone();
                        for y in 1..(screen_size.1 - 1) as usize {
                            for x in 1..(screen_size.0 - 1) as usize {
                                let idx = (y * screen_size.0 as usize + x) * 4;
                                let up = ((y - 1) * screen_size.0 as usize + x) * 4;
                                let down = ((y + 1) * screen_size.0 as usize + x) * 4;
                                let left = (y * screen_size.0 as usize + (x - 1)) * 4;
                                let right = (y * screen_size.0 as usize + (x + 1)) * 4;

                                // Calculate the second-order finite differences for Laplacian (Heat Diffusion)
                                let laplacian = (frame[up] as f32 + frame[down] as f32 + frame[left] as f32 + frame[right] as f32
                                                - 4.0 * frame[idx] as f32) / (delta_x * delta_x);

                                // Update the temperature using the heat equation
                                let new_gray_value = frame[idx] as f32 + alpha * delta_t * laplacian;

                                // Clamp the new value to [0, 255] range
                                let new_clamped_value = new_gray_value.clamp(0.0, 255.0) as u8;

                                // Update the new frame's pixel
                                new_frame[idx] = new_clamped_value;
                                new_frame[idx + 1] = new_clamped_value;
                                new_frame[idx + 2] = new_clamped_value;
                            }
                        }

                        // Copy the new frame back to the current frame
                        frame = new_frame;


                        // Request a redraw
                        window.request_redraw();
                        last_update_inst = now;
                    }
                }
            }
            _ => (),
        }
    });
}
