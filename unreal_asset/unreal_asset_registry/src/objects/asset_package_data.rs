//! Asset bundle asset package data

use byteorder::{ReadBytesExt, WriteBytesExt, LE};

use unreal_asset_base::Guid;

use unreal_asset_base::types::PackageIndexTrait;
use unreal_asset_base::{
    custom_version::{CustomVersion, FAssetRegistryVersionType},
    error::RegistryError,
    reader::{ArchiveReader, ArchiveWriter},
    types::FName,
    Error,
};

use crate::objects::md5_hash::FMD5Hash;

/// Asset package data
#[derive(Debug)]
pub struct AssetPackageData {
    /// Package name
    pub package_name: FName,
    /// Package guid
    pub package_guid: Guid,
    /// Cooked hash
    pub cooked_hash: Option<FMD5Hash>,
    /// Imported classes
    pub imported_classes: Option<Vec<FName>>,
    /// Size on disk
    pub disk_size: i64,
    /// File version
    pub file_version: i32,
    /// UE5 file version
    pub ue5_version: Option<i32>,
    /// File version licensee
    pub file_version_licensee_ue: i32,
    /// Custom versions
    pub custom_versions: Option<Vec<CustomVersion>>,
    /// Flags
    pub flags: u32,
    /// Package build dependencies (UE5.5+)
    pub package_build_dependencies: Option<Vec<FName>>,

    /// Asset registry version
    version: FAssetRegistryVersionType,
}

impl AssetPackageData {
    /// Read `AssetPackageData` from an asset
    pub fn new<Reader: ArchiveReader<impl PackageIndexTrait>>(
        asset: &mut Reader,
        version: FAssetRegistryVersionType,
    ) -> Result<Self, Error> {
        let package_name = asset.read_fname()?;
        let disk_size = asset.read_i64::<LE>()?;

        let package_guid = asset.read_guid()?;

        let mut cooked_hash = None;
        if version >= FAssetRegistryVersionType::AddedCookedMD5Hash {
            cooked_hash = Some(FMD5Hash::new(asset)?);
        }

        let mut file_version = 0;
        let mut ue5_version = None;
        let mut file_version_licensee_ue = -1;
        let mut flags = 0;
        let mut custom_versions = None;
        if version >= FAssetRegistryVersionType::WorkspaceDomain {
            if version >= FAssetRegistryVersionType::PackageFileSummaryVersionChange {
                file_version = asset.read_i32::<LE>()?;
                ue5_version = Some(asset.read_i32::<LE>()?);
            } else {
                file_version = asset.read_i32::<LE>()?;
            }

            file_version_licensee_ue = asset.read_i32::<LE>()?;
            flags = asset.read_u32::<LE>()?;
            custom_versions =
                Some(asset.read_array(|asset: &mut Reader| CustomVersion::read(asset))?);
        }

        let mut imported_classes = None;
        if version >= FAssetRegistryVersionType::PackageImportedClasses {
            imported_classes = Some(asset.read_array(|asset: &mut Reader| asset.read_fname())?);
        }

        // Read package build dependencies if object version is high enough
        let mut package_build_dependencies = None;
        if asset.get_object_version_ue5() >= unreal_asset_base::object_version::ObjectVersionUE5::ASSETREGISTRY_PACKAGEBUILDDEPENDENCIES {
            package_build_dependencies = Some(asset.read_array(|asset: &mut Reader| asset.read_fname())?);
        }

        Ok(Self {
            package_name,
            package_guid,
            cooked_hash,
            imported_classes,
            disk_size,
            file_version,
            ue5_version,
            file_version_licensee_ue,
            custom_versions,
            flags,
            package_build_dependencies,

            version,
        })
    }

    /// Write `AssetPackageData` to an asset
    pub fn write<Writer: ArchiveWriter<impl PackageIndexTrait>>(
        &self,
        asset: &mut Writer,
    ) -> Result<(), Error> {
        asset.write_fname(&self.package_name)?;
        asset.write_i64::<LE>(self.disk_size)?;
        // TODO change to guid method
        asset.write_all(&self.package_guid.0)?;

        if self.version >= FAssetRegistryVersionType::AddedCookedMD5Hash {
            let cooked_hash = self
                .cooked_hash
                .as_ref()
                .ok_or_else(|| RegistryError::version("Cooked hash".to_string(), self.version))?;

            cooked_hash.write(asset)?;
        }

        if self.version >= FAssetRegistryVersionType::WorkspaceDomain {
            if self.version >= FAssetRegistryVersionType::PackageFileSummaryVersionChange {
                asset.write_i32::<LE>(self.file_version)?;
                asset.write_i32::<LE>(self.ue5_version.ok_or_else(|| {
                    RegistryError::version("UE5 Version".to_string(), self.version)
                })?)?;
            } else {
                asset.write_i32::<LE>(self.file_version)?;
            }

            asset.write_i32::<LE>(self.file_version_licensee_ue)?;
            asset.write_u32::<LE>(self.flags)?;

            let custom_versions = self.custom_versions.as_ref().ok_or_else(|| {
                RegistryError::version("Custom versions".to_string(), self.version)
            })?;

            asset.write_i32::<LE>(custom_versions.len() as i32)?;
            for custom_version in custom_versions {
                custom_version.write(asset)?;
            }
        }

        if self.version >= FAssetRegistryVersionType::PackageImportedClasses {
            let imported_classes = self.imported_classes.as_ref().ok_or_else(|| {
                RegistryError::version("Imported classes".to_string(), self.version)
            })?;

            for imported_class in imported_classes {
                asset.write_fname(imported_class)?;
            }
        }

        // Write package build dependencies if the object version is high enough
        if asset.get_object_version_ue5() >= unreal_asset_base::object_version::ObjectVersionUE5::ASSETREGISTRY_PACKAGEBUILDDEPENDENCIES {
            if let Some(build_dependencies) = &self.package_build_dependencies {
                asset.write_i32::<LE>(build_dependencies.len() as i32)?;
                for dependency in build_dependencies {
                    asset.write_fname(dependency)?;
                }
            } else {
                // Write empty array
                asset.write_i32::<LE>(0)?;
            }
        }

        Ok(())
    }
}
