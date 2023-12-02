use std::cmp::max;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use cfg_if::cfg_if;
use platform_dirs::UserDirs;
use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::image::Rgb24;
use crate::pyxel::Pyxel;
use crate::resource_data::ResourceData;
use crate::screencast::Screencast;
use crate::settings::{DEFAULT_CAPTURE_SCALE, DEFAULT_CAPTURE_SEC};
use crate::{PALETTE_FILE_EXTENSION, RESOURCE_ARCHIVE_NAME, RESOURCE_FORMAT_VERSION};

pub struct Resource {
    capture_scale: u32,
    screencast: Screencast,
}

impl Resource {
    pub fn new(capture_scale: Option<u32>, capture_sec: Option<u32>, fps: u32) -> Self {
        let capture_scale = capture_scale.unwrap_or(DEFAULT_CAPTURE_SCALE);
        let capture_sec = capture_sec.unwrap_or(DEFAULT_CAPTURE_SEC);
        Self {
            capture_scale: max(capture_scale, 1),
            screencast: Screencast::new(fps, capture_sec),
        }
    }
}

impl Pyxel {
    pub fn load(
        &mut self,
        filename: &str,
        include_colors: Option<bool>,
        include_images: Option<bool>,
        include_tilemaps: Option<bool>,
        include_channels: Option<bool>,
        include_sounds: Option<bool>,
        include_musics: Option<bool>,
        include_waveforms: Option<bool>,
    ) {
        let mut archive = ZipArchive::new(
            File::open(Path::new(&filename))
                .unwrap_or_else(|_| panic!("Unable to open file '{filename}'")),
        )
        .unwrap_or_else(|_| panic!("Unable to parse zip archive '{filename}'"));

        // Old resource file
        if archive.by_name("pyxel_resource/version").is_ok() {
            self.load_old_resource(
                &mut archive,
                filename,
                include_images.unwrap_or(true),
                include_tilemaps.unwrap_or(true),
                include_sounds.unwrap_or(true),
                include_musics.unwrap_or(true),
            );
            return;
        }

        // New resource file
        let mut file = archive.by_name(RESOURCE_ARCHIVE_NAME).unwrap();
        let mut toml_text = String::new();
        file.read_to_string(&mut toml_text).unwrap();
        let resource_data = ResourceData::from_toml(&toml_text);
        assert!(
            resource_data.format_version > RESOURCE_FORMAT_VERSION,
            "Resource file version is too new"
        );
        resource_data.to_runtime(
            self,
            include_colors.unwrap_or(false),
            include_images.unwrap_or(true),
            include_tilemaps.unwrap_or(true),
            include_channels.unwrap_or(false),
            include_sounds.unwrap_or(true),
            include_musics.unwrap_or(true),
            include_waveforms.unwrap_or(false),
        );

        // Pyxel palette file
        let filename = filename
            .rfind('.')
            .map_or(filename, |i| &filename[..i])
            .to_string()
            + PALETTE_FILE_EXTENSION;
        if let Ok(mut file) = File::open(Path::new(&filename)) {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let colors: Vec<Rgb24> = contents
                .replace("\r\n", "\n")
                .replace('\r', "\n")
                .split('\n')
                .filter(|s| !s.is_empty())
                .map(|s| u32::from_str_radix(s.trim(), 16).unwrap() as Rgb24)
                .collect();
            *self.colors.lock() = colors;
        }
    }

    pub fn save(
        &self,
        filename: &str,
        include_colors: Option<bool>,
        include_images: Option<bool>,
        include_tilemaps: Option<bool>,
        include_channels: Option<bool>,
        include_sounds: Option<bool>,
        include_musics: Option<bool>,
        include_waveforms: Option<bool>,
    ) {
        let toml_text = ResourceData::from_runtime(self).to_toml(
            include_colors.unwrap_or(false),
            include_images.unwrap_or(true),
            include_tilemaps.unwrap_or(true),
            include_channels.unwrap_or(false),
            include_sounds.unwrap_or(true),
            include_musics.unwrap_or(true),
            include_waveforms.unwrap_or(false),
        );
        let path = std::path::Path::new(&filename);
        let file = std::fs::File::create(path)
            .unwrap_or_else(|_| panic!("Unable to open file '{filename}'"));
        let mut zip = ZipWriter::new(file);
        zip.start_file(RESOURCE_ARCHIVE_NAME, FileOptions::default())
            .unwrap();
        zip.write_all(toml_text.as_bytes()).unwrap();
        zip.finish().unwrap();
        #[cfg(target_os = "emscripten")]
        pyxel_platform::emscripten::save_file(filename);
    }

    pub fn screenshot(&self, scale: Option<u32>) {
        let filename = Self::export_path();
        let scale = max(scale.unwrap_or(self.resource.capture_scale), 1);
        self.screen.lock().save(&filename, scale);
        #[cfg(target_os = "emscripten")]
        pyxel_platform::emscripten::save_file(&(filename + ".png"));
    }

    pub fn screencast(&mut self, scale: Option<u32>) {
        let filename = Self::export_path();
        let scale = max(scale.unwrap_or(self.resource.capture_scale), 1);
        self.resource.screencast.save(&filename, scale);
        #[cfg(target_os = "emscripten")]
        pyxel_platform::emscripten::save_file(&(filename + ".gif"));
    }

    pub fn reset_screencast(&mut self) {
        self.resource.screencast.reset();
    }

    pub(crate) fn capture_screen(&mut self) {
        self.resource.screencast.capture(
            self.width,
            self.height,
            &self.screen.lock().canvas.data,
            &self.colors.lock(),
            self.frame_count,
        );
    }

    fn export_path() -> String {
        let desktop_dir = if let Some(user_dirs) = UserDirs::new() {
            user_dirs.desktop_dir
        } else {
            PathBuf::new()
        };
        let basename = "pyxel-".to_string() + &Self::local_time_string();
        desktop_dir.join(basename).to_str().unwrap().to_string()
    }

    fn local_time_string() -> String {
        cfg_if! {
            if #[cfg(target_os = "emscripten")] {
                pyxel_platform::emscripten::timestamp_string()
            } else {
                chrono::Local::now().format("%Y%m%d-%H%M%S").to_string()
            }
        }
    }
}
