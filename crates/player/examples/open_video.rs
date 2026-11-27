fn main() -> Result<(), Box<dyn std::error::Error>> {
    let player = rod_player::Player::open("testdata/clip-1080p.mp4")?;
    let mut flags = 0;
    for _ in 0..100 {
        flags = player.update();
        let mut buf = vec![0u8; 1920 * 1080 * 4];
        player.render(&mut buf, 1920, 1080)?;
        let distintos = buf.iter().filter(|b| **b != 0).count();
        println!("bytes distintos de cero: {distintos} de {}", buf.len());
        println!("primer píxel: {:?}", &buf[0..4]);
        let mut rgb = Vec::with_capacity(1920 * 1080 * 3);
        for px in buf.as_chunks::<4>().0 {
            rgb.extend_from_slice(&px[0..3]);
        }
        image::RgbImage::from_raw(1920, 1080, rgb)
            .unwrap()
            .save("testdata/frame.png")?;
        if flags != 0 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    println!("flags = {flags}");
    Ok(())
}
