use core::f32::consts::PI;

pub fn make_1900ish(input: &[f32], sample_rate: f32) -> Vec<f32> {
    if input.is_empty() || sample_rate <= 0.0 {
        return Vec::new();
    }

    // さらに狭い帯域（初期アコースティック録音寄り）
    let hp_cut = 420.0;
    let lp_cut = 2500.0;
    let hp_a = (-2.0 * PI * hp_cut / sample_rate).exp();
    let lp_a = (-2.0 * PI * lp_cut / sample_rate).exp();

    let mut out = Vec::with_capacity(input.len());
    let mut hp_y = 0.0;
    let mut lp_y = 0.0;
    let mut hp_x_prev = 0.0;

    let mut noise_state: u32 = 0x1234_abcd;
    let mut crackle_env = 0.0;

    for (i, &x) in input.iter().enumerate() {
        let t = i as f32 / sample_rate;

        // 重めの wow / flutter
        let wow = (2.0 * PI * 0.35 * t).sin() * 0.05;
        let flutter = (2.0 * PI * 4.5 * t).sin() * 0.015;
        let am = 1.0 + wow + flutter;

        // DC バイアス（非対称歪みを強調）
        let biased = x * am + 0.02;

        // HP → LP
        let hp = hp_a * (hp_y + biased - hp_x_prev);
        hp_x_prev = biased;
        hp_y = hp;

        let lp = (1.0 - lp_a) * hp + lp_a * lp_y;
        lp_y = lp;

        // 二段歪み：ホーン的 → カッティング的
        let y1 = soft_clip(lp * 2.0);
        let y2 = asymmetric_clip(y1 * 1.4);

        // ランブル（78rpm系）
        let rumble =
            (2.0 * PI * 28.0 * t).sin() * 0.003 +
            (2.0 * PI * 55.0 * t).sin() * 0.0015;

        // ヒス
        let hiss = lcg_noise(&mut noise_state) * 0.004;

        // クラックル（ランダムに発生する減衰パルス）
        if lcg_noise(&mut noise_state) > 0.995 {
            crackle_env = 1.0;
        }
        crackle_env *= 0.94;
        let crackle = crackle_env * (lcg_noise(&mut noise_state) * 0.08);

        let mut y = y2 + rumble + hiss + crackle;

        // 粗い量子化
        y = quantize(y, 9);

        out.push(y.clamp(-1.0, 1.0));
    }

    out
}

fn soft_clip(x: f32) -> f32 {
    x / (1.0 + x.abs())
}

// 非対称クリップ（機械系の偏り）
fn asymmetric_clip(x: f32) -> f32 {
    if x >= 0.0 {
        x / (1.0 + x)
    } else {
        x / (1.0 + 0.5 * x.abs())
    }
}

fn quantize(x: f32, bits: u32) -> f32 {
    let levels = (1u32 << bits) as f32;
    (x * levels).round() / levels
}

fn lcg_noise(state: &mut u32) -> f32 {
    *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
    let v = *state >> 9;
    (v as f32 / (1u32 << 23) as f32) * 2.0 - 1.0
}
