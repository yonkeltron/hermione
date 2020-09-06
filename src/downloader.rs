use anyhow::Result;
use paris::Logger;
use reqwest;
use tempfile::Builder;

use std::fs;
use std::io::copy;
use std::path::PathBuf;

use crate::downloaded_package::DownloadedPackage;
use crate::package_service::PackageService;
use crate::packer::Packer;

/// Represents the data required to download a repo from a git server.
pub struct Downloader {
    remote_package_path: String,
    package_service: PackageService,
}

impl Downloader {
    pub fn new(remote_package_path: String, package_service: PackageService) -> Self {
        Self {
            remote_package_path,
            package_service,
        }
    }

    fn download_file(&self) -> Result<PathBuf> {
        let mut logger = Logger::new();

        logger.info(format!(
            "Downlaoding hermione package from {}",
            &self.remote_package_path
        ));
        let tmp_dir = Builder::new().prefix("hermione_pkg").tempdir()?;
        let response = reqwest::blocking::get(&self.remote_package_path)?;
        let fname = response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("tmp.bin");

        logger
            .indent(1)
            .info(format!("File to download: {}", &fname));

        let fpath = tmp_dir.path().join(fname);

        let mut dest = fs::File::create(&fpath)?;
        let content = response.text()?;
        copy(&mut content.as_bytes(), &mut dest)?;

        logger
            .indent(1)
            .info(format!("File Temporarly saved to: {}", &fpath.display()));

        Ok(fpath)
    }

    pub fn download(self) -> Result<DownloadedPackage> {
        let file_path_buf = self.download_file()?;
        let unpacked_archive_path =
            Packer::new(file_path_buf).unpack(self.package_service.download_dir())?;
        Ok(DownloadedPackage {
            local_path: unpacked_archive_path,
            package_service: self.package_service,
        })
    }
}
