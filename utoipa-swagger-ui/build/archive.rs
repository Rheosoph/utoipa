use std::{
    fs,
    io::{self, Read, Seek},
    path::PathBuf,
};

use zip::{result::ZipError, ZipArchive};

pub(crate) fn extract_dist<R: Seek + Read>(
    zip: &mut ZipArchive<R>,
    target_dir: &str,
) -> Result<String, ZipError> {
    zip.validate_header_consistency()?;
    let mut zip_top_level_folder = String::new();

    for index in 0..zip.len() {
        let mut file = zip.by_index(index)?;
        let filepath = file
            .enclosed_name()
            .ok_or(ZipError::InvalidArchive("invalid path file".into()))?;

        if index == 0 {
            zip_top_level_folder = filepath
                .iter()
                .take(1)
                .map(|x| x.to_str().unwrap_or_default())
                .collect::<String>();
        }

        let next_folder = filepath
            .iter()
            .skip(1)
            .take(1)
            .map(|x| x.to_str().unwrap_or_default())
            .collect::<String>();

        if next_folder == "dist" {
            let directory = [&target_dir].iter().collect::<PathBuf>();
            let out_path = directory.join(filepath);

            if file.is_dir() {
                fs::create_dir_all(&out_path)?;
            } else {
                if let Some(p) = out_path.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p)?;
                    }
                }
                let mut out_file = fs::File::create(&out_path)?;
                io::copy(&mut file, &mut out_file)?;
            }
            // Get and Set permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&out_path, fs::Permissions::from_mode(mode))?;
                }
            }
        }
    }

    Ok(zip_top_level_folder)
}
