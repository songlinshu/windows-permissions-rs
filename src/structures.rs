pub use sd::SecurityDescriptor;
pub use sid::Sid;

mod sid {
    use crate::wrappers;
    use std::fmt;
    use std::ptr::NonNull;
    use winapi::um::winnt::SID;

    #[repr(C)]
    pub struct Sid(SID);

    impl Sid {
        /// Get `&Sid` from a `NonNull<SID>`
        ///
        /// The `_lifetime` parameter indicates the lifetime of the reference.
        ///
        /// ## Requirements
        /// - `ptr` points to a valid SID
        /// - `_lifetime` lives at least as long as `ptr`
        pub unsafe fn from_nonnull<T>(ptr: NonNull<SID>, _lifetime: &T) -> &Sid {
            &*(ptr.as_ptr() as *mut Sid)
        }
    }

    impl fmt::Debug for Sid {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            fmt.debug_map()
                .entry(&"rev", &self.0.Revision)
                .entry(&"sub_auth_count", &self.0.SubAuthorityCount)
                .entry(&"id_auth", &self.0.IdentifierAuthority.Value)
                .entry(&"sub_auth", &self.0.SubAuthority)
                .finish()
        }
    }

    impl PartialEq for Sid {
        fn eq(&self, other: &Sid) -> bool {
            wrappers::EqualSid(self, other)
        }
    }
}

mod sd {
    use std::ptr::NonNull;
    use winapi::um::winnt::{ACL, PACL, PSECURITY_DESCRIPTOR, PSID, SECURITY_DESCRIPTOR, SID};

    use super::sid::Sid;

    pub struct SecurityDescriptor {
        sd: NonNull<SECURITY_DESCRIPTOR>,
        owner: Option<NonNull<SID>>,
        group: Option<NonNull<SID>>,
        dacl: Option<NonNull<ACL>>,
        sacl: Option<NonNull<ACL>>,
    }

    impl SecurityDescriptor {
        /// Construct a security descriptor from raw parts
        ///
        /// ## Assumptions
        ///
        /// - `sd` points to a valid buffer and should be deallocated with
        ///   `LocalFree`
        /// - All of the other pointers are either null or point at something
        ///   in the `sd` buffer
        /// - The two `PSID` arguments point to valid SID structures and the
        ///   two `ACL` arguments point to valid ACL structures
        ///
        /// ## Panics
        ///
        /// Panics if `sd` is null.
        pub unsafe fn from_raw(
            sd: PSECURITY_DESCRIPTOR,
            owner: PSID,
            group: PSID,
            dacl: PACL,
            sacl: PACL,
        ) -> SecurityDescriptor {
            SecurityDescriptor {
                sd: NonNull::new(sd as *mut SECURITY_DESCRIPTOR)
                    .expect("SecurityDescriptor::from_raw called with null sd pointer"),
                owner: NonNull::new(owner as *mut SID),
                group: NonNull::new(group as *mut SID),
                dacl: NonNull::new(dacl),
                sacl: NonNull::new(sacl),
            }
        }

        /// Get the owner SID if it exists
        pub fn owner(&self) -> Option<&Sid> {
            // Assumptions:
            // - self.owner lives as long as self
            self.owner
                .clone()
                .map(|p| unsafe { Sid::from_nonnull(p, self) })
        }

        /// Get the group SID if it exists
        pub fn group(&self) -> Option<&Sid> {
            // Assumptions:
            // - self.group lives as long as self
            self.group
                .clone()
                .map(|p| unsafe { Sid::from_nonnull(p, self) })
        }
    }

    impl Drop for SecurityDescriptor {
        fn drop(&mut self) {
            let result = unsafe { winapi::um::winbase::LocalFree(self.sd.as_ptr() as *mut _) };
            assert!(result.is_null());
        }
    }
}
