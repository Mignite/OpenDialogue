use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let player = rod_player::Player::open("testdata/clip-1080p.mp4")?;
    player.mpv().set_property("loop-file", "inf".to_string())?;
    player.mpv().set_property("untimed", true)?;

    let mut buf = vec![0u8; 1920 * 1080 * 4];
    let mut tiempos = Vec::with_capacity(100);

    while tiempos.len() < 100 {
        if player.update() & 1 == 0 {
            std::thread::sleep(Duration::from_millis(1));
            continue;
        }
        let t0 = Instant::now();
        player.render(&mut buf, 1920, 1080)?;
        tiempos.push(t0.elapsed());
    }

    let mut ms: Vec<f64> = tiempos.iter().map(|d| d.as_secs_f64() * 1000.0).collect();
    ms.sort_by(f64::total_cmp);
    let media = ms.iter().sum::<f64>() / ms.len() as f64;
    println!(
        "media {media:.2} ms   p95 {:.2} ms   fps {:.1}",
        ms[94],
        1000.0 / media
    );
    Ok(())
}
