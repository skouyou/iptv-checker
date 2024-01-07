use crate::lib::{AudioInfo, VideoInfo};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckUrlIsAvailableResponse {
    pub(crate) delay: i32,
    pub(crate) video: Option<VideoInfo>,
    pub(crate) audio: Option<AudioInfo>,
}

impl CheckUrlIsAvailableResponse {
    pub fn new() -> CheckUrlIsAvailableResponse {
        CheckUrlIsAvailableResponse {
            delay: 0,
            video: None,
            audio: None,
        }
    }

    pub fn set_delay(&mut self, delay: i32) {
        self.delay = delay
    }

    pub fn set_video(&mut self, video: VideoInfo) {
        self.video = Some(video)
    }

    pub fn set_audio(&mut self, audio: AudioInfo) {
        self.audio = Some(audio)
    }
}

// #[derive(Serialize, Deserialize)]
// pub struct CheckUrlIsAvailableRespAudio {
//     pub(crate) codec: String,
//     pub(crate) channels: i32,
//     #[serde(rename = "bitRate")]
//     pub(crate) bit_rate: i32,
// }

// impl CheckUrlIsAvailableRespAudio {
//     pub fn new() -> CheckUrlIsAvailableRespAudio {
//         CheckUrlIsAvailableRespAudio {
//             codec: "".to_string(),
//             channels: 0,
//             bit_rate: 0,
//         }
//     }
//
//     pub fn set_codec(&mut self, codec: String) {
//         self.codec = codec
//     }
//
//     pub fn set_channels(&mut self, channels: i32) {
//         self.channels = channels
//     }
//     pub fn set_bit_rate(&mut self, bit_rate: i32) {
//         self.bit_rate = bit_rate
//     }
//
//     pub fn get_bit_rate(self) -> i32 {
//         self.bit_rate
//     }
//     pub fn get_channels(self) -> i32 {
//         self.channels
//     }
//     pub fn get_codec(self) -> String {
//         self.codec
//     }
// }

// #[derive(Serialize, Deserialize)]
// pub struct CheckUrlIsAvailableRespVideo {
//     width: i32,
//     height: i32,
//     codec: String,
//     #[serde(rename = "bitRate")]
//     bit_rate: i32,
// }

#[derive(Debug, Deserialize, Serialize)]
pub struct Ffprobe {
    streams: Vec<FfprobeStream>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FfprobeStream {
    codec_type: String,
    width: Option<i32>,
    height: Option<i32>,
    codec_name: String,
    channels: Option<i32>,
}

pub mod check {
    use crate::lib::{AudioInfo, CheckUrlIsAvailableResponse, Ffprobe, VideoInfo};
    use chrono::Utc;
    use std::io::{Error, ErrorKind};
    use std::process::Command;
    use std::time;
    use futures::future::err;

    pub async fn check_link_is_valid(
        _url: String,
        timeout: u64,
    ) -> Result<CheckUrlIsAvailableResponse, Error> {
        let client = reqwest::Client::builder()
            .timeout(time::Duration::from_millis(timeout))
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();
        let curr_timestamp = Utc::now().timestamp_millis();
        let check_data = client.get(_url.to_owned()).send().await;
        match check_data {
            Ok(res) => {
                if res.status().is_success() {
                    let mut ffprobe = Command::new("ffprobe");
                    let mut prob = ffprobe
                        .arg("-v")
                        .arg("quiet")
                        .arg("-print_format")
                        .arg("json");
                    if timeout > 0 {
                        prob = prob.arg("-timeout").arg(timeout.to_string());
                    }
                    let prob_result = prob
                        .arg("-show_format")
                        .arg("-show_streams")
                        .arg(_url.to_owned())
                        .output()
                        .unwrap();
                    if prob_result.status.success() {
                        let res_data: Ffprobe = serde_json::from_str(
                            String::from_utf8(prob_result.stdout).unwrap().as_str(),
                        )
                        .expect("无法解析 JSON");
                        let delay = Utc::now().timestamp_millis() - curr_timestamp;
                        let mut body: CheckUrlIsAvailableResponse =
                            CheckUrlIsAvailableResponse::new();
                        body.set_delay(delay as i32);
                        for one in res_data.streams.into_iter() {
                            if one.codec_type == "video" {
                                let mut video = VideoInfo::new();
                                match one.width {
                                    Some(e) => video.set_width(e),
                                    _ => {}
                                }
                                match one.height {
                                    Some(e) => video.set_height(e),
                                    _ => {}
                                }
                                video.set_codec(one.codec_name);
                                body.set_video(video);
                            } else if one.codec_type == "audio" {
                                let mut audio = AudioInfo::new();
                                audio.set_codec(one.codec_name);
                                audio.set_channels(one.channels.unwrap());
                                body.set_audio(audio);
                            }
                        }
                        return Ok(body);
                    } else {
                        let error_str = String::from_utf8_lossy(&prob_result.stderr);
                        println!("ffprobe error {:?}", prob_result.stderr);
                        return Err(Error::new(ErrorKind::Other, error_str.to_string()));
                    }
                }
                return Err(Error::new(ErrorKind::Other, "status is not 200"));
            }
            Err(e) => {
                println!("http request error : {}", e);
                return Err(Error::new(ErrorKind::Other, e));
            }
        };
    }

    pub fn check_can_support_ipv6() -> Result<bool, Error> {
        // curl -6 test.ipw.cn
        Ok(true)
    }
}
