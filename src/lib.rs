use aviutl2::filter::{FilterConfigItemSliceExt, FilterConfigItems};

mod vintage;

#[aviutl2::plugin(FilterPlugin)]
#[derive(Debug)]
struct VintageAuf {}

#[aviutl2::filter::filter_config_items]
struct Config {
    #[track(name = "Wet", range = 0..=100, default = 100, step = 0.1)]
    wet: u8,
}

impl aviutl2::filter::FilterPlugin for VintageAuf {
    fn new(_info: aviutl2::AviUtl2Info) -> aviutl2::AnyResult<VintageAuf> {
        Ok(Self {})
    }

    fn plugin_info(&self) -> aviutl2::filter::FilterPluginTable {
        aviutl2::filter::FilterPluginTable {
            name: "vintage.auf2".to_string(),
            label: Some("加工".to_string()),
            information: format!(
                "Vintage-ish audio effect, written in Rust / v{version} / https://github.com/sevenc-nanashi/vintage.auf2",
                version = env!("CARGO_PKG_VERSION")
            ),
            flags: aviutl2::bitflag!(aviutl2::filter::FilterPluginFlags {
                audio: true,
                as_filter: true
            }),
            config_items: Config::to_config_items(),
        }
    }

    fn proc_audio(
        &self,
        config: &[aviutl2::filter::FilterConfigItem],
        audio: &mut aviutl2::filter::FilterProcAudio,
    ) -> aviutl2::AnyResult<()> {
        let config = config.to_struct::<Config>();
        let mut lbuf = vec![0.0_f32; audio.audio_object.sample_num as usize];
        let mut rbuf = vec![0.0_f32; audio.audio_object.sample_num as usize];
        audio.get_sample_data(aviutl2::filter::AudioChannel::Left, &mut lbuf);
        audio.get_sample_data(aviutl2::filter::AudioChannel::Right, &mut rbuf);
        let buf = if audio.audio_object.channel_num == 1 {
            lbuf.clone()
        } else {
            lbuf.iter()
                .zip(rbuf.iter())
                .map(|(&l, &r)| (l + r) * 0.5)
                .collect::<Vec<f32>>()
        };
        let processed = vintage::make_1900ish(&buf, audio.scene.sample_rate as f32);
        let wet = config.wet as f32 / 100.0;

        if audio.audio_object.channel_num == 1 {
            let mut out_buf = vec![0.0_f32; audio.audio_object.sample_num as usize];
            for (i, sample) in processed.iter().enumerate() {
                out_buf[i] = buf[i] * (1.0 - wet) + sample * wet;
            }
            audio.set_sample_data(aviutl2::filter::AudioChannel::Left, &out_buf);
        } else {
            let mut out_buf_l = vec![0.0_f32; audio.audio_object.sample_num as usize];
            let mut out_buf_r = vec![0.0_f32; audio.audio_object.sample_num as usize];
            for (i, sample) in processed.iter().enumerate() {
                out_buf_l[i] = lbuf[i] * (1.0 - wet) + sample * wet;
                out_buf_r[i] = rbuf[i] * (1.0 - wet) + sample * wet;
            }
            audio.set_sample_data(aviutl2::filter::AudioChannel::Left, &out_buf_l);
            audio.set_sample_data(aviutl2::filter::AudioChannel::Right, &out_buf_r);
        }
        Ok(())
    }
}

aviutl2::register_filter_plugin!(VintageAuf);
