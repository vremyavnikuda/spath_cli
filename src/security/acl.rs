use anyhow::{Context, Result};
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use tracing::{debug, info};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{LocalFree, HLOCAL, PSID};
use windows::Win32::Security::Authorization::{
    SetEntriesInAclW, SetNamedSecurityInfoW, EXPLICIT_ACCESS_W, SET_ACCESS, SE_FILE_OBJECT,
    TRUSTEE_IS_SID, TRUSTEE_TYPE, TRUSTEE_W,
};
use windows::Win32::Security::{
    GetTokenInformation, TokenUser, ACE_FLAGS, ACL, DACL_SECURITY_INFORMATION,
    PROTECTED_DACL_SECURITY_INFORMATION, TOKEN_QUERY, TOKEN_USER,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

/// Sets ACL on a file to allow access only to the current user.
///
/// This function:
/// 1. Gets the current user's SID
/// 2. Creates a DACL with a single ACE granting full control to the current user
/// 3. Applies the DACL to the specified file
///
/// # Security
/// - Removes all inherited permissions
/// - Grants full control only to the current user
/// - Protects against unauthorized access to backup files
pub fn set_user_only_acl(path: &Path) -> Result<()> {
    debug!("Setting user-only ACL for: {}", path.display());
    if !path.exists() {
        anyhow::bail!("File does not exist: {}", path.display());
    }
    let canonical_path = path.canonicalize().context("Failed to canonicalize path")?;
    let (_buffer, user_sid) = get_current_user_sid()?;
    let ea = EXPLICIT_ACCESS_W {
        grfAccessPermissions: 0x1F01FF,
        grfAccessMode: SET_ACCESS,
        grfInheritance: ACE_FLAGS(0),
        Trustee: TRUSTEE_W {
            pMultipleTrustee: std::ptr::null_mut(),
            MultipleTrusteeOperation: Default::default(),
            TrusteeForm: TRUSTEE_IS_SID,
            TrusteeType: TRUSTEE_TYPE(1),
            ptstrName: windows::core::PWSTR(user_sid.0 as *mut u16),
        },
    };
    let mut acl: *mut ACL = std::ptr::null_mut();
    unsafe {
        SetEntriesInAclW(Some(&[ea]), None, &mut acl)
            .context("Failed to create ACL with user permissions")?;
        let path_wide: Vec<u16> = canonical_path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let result = SetNamedSecurityInfoW(
            PCWSTR(path_wide.as_ptr()),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            None,
            None,
            Some(acl),
            None,
        );
        if !acl.is_null() {
            let _ = LocalFree(HLOCAL(acl as *mut _));
        }
        result.context("Failed to set security info on file")?;
    }
    info!("Successfully set user-only ACL for: {}", path.display());
    Ok(())
}

/// Gets the SID of the current user.
/// Returns a buffer containing the TOKEN_USER structure and the SID.
/// The PSID points into this buffer, so the buffer must be kept alive.
fn get_current_user_sid() -> Result<(Vec<u8>, PSID)> {
    debug!("Getting current user SID");
    unsafe {
        let mut token = Default::default();
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token)
            .context("Failed to open process token")?;
        let mut size = 0u32;
        let _ = GetTokenInformation(token, TokenUser, None, 0, &mut size);
        let mut buffer = vec![0u8; size as usize];
        GetTokenInformation(
            token,
            TokenUser,
            Some(buffer.as_mut_ptr() as *mut _),
            size,
            &mut size,
        )
        .context("Failed to get token information")?;
        let token_user = buffer.as_ptr() as *const TOKEN_USER;
        let sid = (*token_user).User.Sid;
        debug!("Successfully retrieved current user SID");
        Ok((buffer, sid))
    }
}
