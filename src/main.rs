#![allow(dead_code)]
#![allow(unused_variables)]

mod render;

use std::error::Error;
use std::time::Instant;
use std::io::{stdout, Write, BufWriter};

use image::{ImageBuffer, Rgba};

use crossterm::{ExecutableCommand, QueueableCommand, terminal::{Clear, ClearType},
    style::{self, SetAttribute, Color, Attribute},
    cursor, terminal
};

//use ansi_term::Colour::RGB;

fn get_ascii(pixel: Rgba<u8>) -> char {
    if pixel[3] == 0 {
        ' '
    }
    else {
        '0'
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut stdout = stdout();
    
    stdout.execute(Clear(ClearType::All))?;
    stdout.execute(cursor::Hide)?;

    let (cols, rows) = terminal::size()?;

    let width = cols as u32;
    let height = rows as u32;

    let (models, _materials) = tobj::load_obj("suzanne.obj", false)?;
    let mesh = &models[0].mesh;
    let vertices = mesh.positions.iter().cloned();
    let normals = mesh.normals.iter().cloned();
    let indices = mesh.indices.iter().cloned();

    let renderer = render::Renderer::new()?;
    let mut pipeline = render::Pipeline::new(renderer, width, height, vertices, normals, indices)?;

    let blank_image = vec![0 as u8; (height * width * 4) as usize];
    let blank_buffer = ImageBuffer::<Rgba<u8>, Vec<u8>>::
        from_raw(
            width, 
            height, 
            blank_image
        ).unwrap();
    let mut swapchain: Vec<ImageBuffer::<Rgba<u8>, Vec<u8>>> = vec![blank_buffer; 2];

    let mut i = 0;
    let rotation_start = Instant::now();
    let mut frame_average = 0.0;
     
    loop {
        let frame_duration = Instant::now();
        let mut stdout_lock = stdout.lock();
        i = 1 - i;
        let new_image = pipeline.render(rotation_start.elapsed())?.clone();
        swapchain[i] = ImageBuffer::<Rgba<u8>, Vec<u8>>::
        from_raw(
            width, 
            height, 
            new_image
        ).unwrap();

        for (x, y, pixel) in swapchain[i].enumerate_pixels() {
            if *swapchain[1 - i].get_pixel(x, y) != *pixel {
                stdout_lock
                    .queue(cursor::MoveTo(x as u16, y as u16))?
                    .queue(style::SetForegroundColor(Color::Rgb { r: pixel[0], g: pixel[1], b: pixel[2] }))?
                    .queue(style::Print(get_ascii(*pixel)))?;
            }
        }
        frame_average = frame_average * 0.95 + frame_duration.elapsed().as_millis() as f32 * 0.05;
        stdout_lock
            .queue(cursor::MoveTo(0, 0))?
            .queue(style::SetForegroundColor(Color::Rgb { r: 255, g: 0, b: 0 }))?
            .queue(style::Print(1.0 / (frame_average / 1000.0)))?;
    }
       
/*
    loop {
        let frame_duration = Instant::now();
        let mut output_text = Vec::<u8>::new();
        //let mut stdout_lock = stdout.lock();
        let new_image = pipeline.render(rotation_start.elapsed())?.clone();
        swapchain[i] = ImageBuffer::<Rgba<u8>, Vec<u8>>::
        from_raw(
            width, 
            height, 
            new_image
        ).unwrap();
        for (x, y, pixel) in swapchain[i].enumerate_pixels() {
            output_text
                .queue(cursor::MoveTo(x as u16, y as u16))?
                .queue(style::SetForegroundColor(Color::Rgb { r: pixel[0], g: pixel[1], b: pixel[2] }))?
                .queue(style::Print(get_ascii(*pixel)))?;
        }
        frame_average = frame_average * 0.9 + frame_duration.elapsed().as_millis() as f32 * 0.1;
        output_text
            .queue(cursor::MoveTo(0, 0))?
            .queue(style::SetForegroundColor(Color::Rgb { r: 255, g: 0, b: 0 }))?
            .queue(style::Print(1.0 / (frame_average / 1000.0)))?;
        std::io::copy(&mut &output_text[..], &mut stdout)?;
        //stdout_lock.write_all(&output_text)?;
    }
    
    stdout.queue(SetAttribute(Attribute::Reset))?;
    Ok(())
    */
}
