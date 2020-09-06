use anyhow::Result;
use paris::Logger;
use reqwest;
use tempfile::Builder;

use std::fs;
use std::io::copy;

use crate::downloaded_package::DownloadedPackage;
use crate::package_service::PackageService;
use crate::packer::Packer;

/// Represents the data required to download a package from a remote server.
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

    pub fn download(self) -> Result<DownloadedPackage> {
        let mut logger = Logger::new();

        logger.info(format!(
            "Downlaoding hermione package from {}",
            &self.remote_package_path
        ));

        let tmp_dir = Builder::new().prefix("hermione_pkg_").tempdir()?;
        let mut response = reqwest::blocking::get(&self.remote_package_path)?;
        let fname = response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("tmp.bin");

        logger
            .indent(1)
            .log(format!("File to download: {}", &fname));

        let file_path_buf = tmp_dir.path().join(fname);

        let mut dest = fs::File::create(&file_path_buf)?;
        copy(&mut response, &mut dest)?;

        logger.indent(1).log(format!(
            "File Temporarly saved to: {}",
            file_path_buf.display()
        ));

        let unpacked_archive_path =
            Packer::new(file_path_buf).unpack(self.package_service.download_dir())?;

        Ok(DownloadedPackage {
            local_path: unpacked_archive_path,
            package_service: self.package_service,
        })
    }
}
