use std::error::Error;
use std::ffi::CString;
use std::io;

use crate::disk::UsageStat;

pub fn get_usage(path: &str) -> Result<UsageStat, Box<dyn Error>> {
    let stat = statfs(path)?;


    let mut ret = UsageStat {
        path: path.to_string(),
        fs_type: get_fs_type(stat.f_type as isize),
        total: stat.f_blocks as u64 * stat.f_bsize as u64,
        free: stat.f_bavail as u64 * stat.f_bsize as u64,
        inodes_total: stat.f_files as u64,
        inodes_free: stat.f_ffree as u64,
        used: (stat.f_blocks as u64 - stat.f_bfree as u64) * stat.f_bsize as u64,
        ..Default::default()
    };

    if ret.used + ret.free == 0 {
        ret.used_percent = 0.0;
    } else {
        ret.used_percent = (ret.used as f64 / (ret.used + ret.free) as f64) * 100.0;
    }

    if ret.inodes_total < ret.inodes_free {
        return Ok(ret);
    }


    ret.inodes_used = ret.inodes_total - ret.inodes_free;
    if ret.inodes_total == 0 {
        ret.inodes_used_percent = 0.0;
    } else {
        ret.inodes_used_percent = (ret.inodes_used as f64 / ret.inodes_total as f64) * 100.0;
    }

    Ok(ret)
}

fn get_fs_type(type_id: isize) -> String {
    match type_id {
        0xadf5 => "adfs",          // ADFS_SUPER_MAGIC      /* 0xADF5 local */
        0xADFF => "affs",          // AFFS_SUPER_MAGIC      /* 0xADFF local */
        0x5346414F => "afs",       // AFS_SUPER_MAGIC       /* 0x5346414F remote */
        0x09041934 => "anon-inode FS", // ANON_INODE_FS_SUPER_MAGIC /* 0x09041934 local */
        0x61756673 => "aufs",      // AUFS_SUPER_MAGIC      /* 0x61756673 remote */
        0x42465331 => "befs",      // BEFS_SUPER_MAGIC      /* 0x42465331 local */
        0x62646576 => "bdevfs",    // BDEVFS_MAGIC          /* 0x62646576 local */
        0x1BADFACE => "bfs",       // BFS_MAGIC             /* 0x1BADFACE local */
        0x42494E4D => "binfmt_misc", // BINFMTFS_MAGIC      /* 0x42494E4D local */
        0xCAFE4A11 => "bpf",       // BPF_FS_MAGIC          /* 0xCAFE4A11 local */
        0x9123683E => "btrfs",     // BTRFS_SUPER_MAGIC     /* 0x9123683E local */
        0x00C36400 => "ceph",      // CEPH_SUPER_MAGIC      /* 0x00C36400 remote */
        0x0027E0EB => "cgroupfs",  // CGROUP_SUPER_MAGIC    /* 0x0027E0EB local */
        0x63677270 => "cgroup2fs", // CGROUP2_SUPER_MAGIC   /* 0x63677270 local */
        0xFF534D42 => "cifs",      // CIFS_MAGIC_NUMBER     /* 0xFF534D42 remote */
        0x73757245 => "coda",      // CODA_SUPER_MAGIC      /* 0x73757245 remote */
        0x012FF7B7 => "coh",       // COH_SUPER_MAGIC       /* 0x012FF7B7 local */
        0x62656570 => "configfs",  // CONFIGFS_MAGIC        /* 0x62656570 local */
        0x28CD3D45 => "cramfs",    // CRAMFS_MAGIC          /* 0x28CD3D45 local */
        0x64626720 => "debugfs",   // DEBUGFS_MAGIC         /* 0x64626720 local */
        0x1373 => "devfs",         // DEVFS_SUPER_MAGIC     /* 0x1373 local */
        0x1CD1 => "devpts",        // DEVPTS_SUPER_MAGIC    /* 0x1CD1 local */
        0xF15F => "ecryptfs",      // ECRYPTFS_SUPER_MAGIC  /* 0xF15F local */
        0xDE5E81E4 => "efivarfs",  // EFIVARFS_MAGIC        /* 0xDE5E81E4 local */
        0x00414A53 => "efs",       // EFS_SUPER_MAGIC       /* 0x00414A53 local */
        0x137D => "ext",           // EXT_SUPER_MAGIC       /* 0x137D local */
        0xEF53 => "ext2/ext3",     // EXT2_SUPER_MAGIC      /* 0xEF53 local */
        0xEF51 => "ext2",          // EXT2_OLD_SUPER_MAGIC  /* 0xEF51 local */
        0xF2F52010 => "f2fs",      // F2FS_SUPER_MAGIC      /* 0xF2F52010 local */
        0x4006 => "fat",           // FAT_SUPER_MAGIC       /* 0x4006 local */
        0x19830326 => "fhgfs",     // FHGFS_SUPER_MAGIC     /* 0x19830326 remote */
        0x65735546 => "fuseblk",   // FUSEBLK_SUPER_MAGIC   /* 0x65735546 remote */
        0x65735543 => "fusectl",   // FUSECTL_SUPER_MAGIC   /* 0x65735543 remote */
        0x0BAD1DEA => "futexfs",   // FUTEXFS_SUPER_MAGIC   /* 0x0BAD1DEA local */
        0x1161970 => "gfs/gfs2",   // GFS_SUPER_MAGIC       /* 0x1161970 remote */
        0x47504653 => "gpfs",      // GPFS_SUPER_MAGIC      /* 0x47504653 remote */
        0x4244 => "hfs",           // HFS_SUPER_MAGIC       /* 0x4244 local */
        0x482b => "hfsplus",       // HFSPLUS_SUPER_MAGIC   /* 0x482b local */
        0xF995E849 => "hpfs",      // HPFS_SUPER_MAGIC      /* 0xF995E849 local */
        0x958458F6 => "hugetlbfs", // HUGETLBFS_MAGIC       /* 0x958458F6 local */
        0x11307854 => "inodefs",   // MTD_INODE_FS_SUPER_MAGIC /* 0x11307854 local */
        0x2BAD1DEA => "inotifyfs", // INOTIFYFS_SUPER_MAGIC /* 0x2BAD1DEA local */
        0x9660 => "isofs",         // ISOFS_SUPER_MAGIC     /* 0x9660 local */
        0x4004 => "isofs",         // ISOFS_R_WIN_SUPER_MAGIC /* 0x4004 local */
        0x4000 => "isofs",         // ISOFS_WIN_SUPER_MAGIC /* 0x4000 local */
        0x07C0 => "jffs",          // JFFS_SUPER_MAGIC      /* 0x07C0 local */
        0x72B6 => "jffs2",         // JFFS2_SUPER_MAGIC     /* 0x72B6 local */
        0x3153464A => "jfs",       // JFS_SUPER_MAGIC       /* 0x3153464A local */
        0x6B414653 => "k-afs",     // KAFS_SUPER_MAGIC      /* 0x6B414653 remote */
        0x0BD00BD0 => "lustre",    // LUSTRE_SUPER_MAGIC    /* 0x0BD00BD0 remote */
        0x137F => "minix",         // MINIX_SUPER_MAGIC     /* 0x137F local */
        0x138F => "minix (30 char.)", // MINIX_SUPER_MAGIC2 /* 0x138F local */
        0x2468 => "minix v2",      // MINIX2_SUPER_MAGIC    /* 0x2468 local */
        0x2478 => "minix v2 (30 char.)", // MINIX2_SUPER_MAGIC2 /* 0x2478 local */
        0x4D5A => "minix3",        // MINIX3_SUPER_MAGIC    /* 0x4D5A local */
        0x19800202 => "mqueue",    // MQUEUE_MAGIC          /* 0x19800202 local */
        0x4D44 => "msdos",         // MSDOS_SUPER_MAGIC     /* 0x4D44 local */
        0x564C => "novell",        // NCP_SUPER_MAGIC       /* 0x564C remote */
        0x6969 => "nfs",           // NFS_SUPER_MAGIC       /* 0x6969 remote */
        0x6E667364 => "nfsd",      // NFSD_SUPER_MAGIC      /* 0x6E667364 remote */
        0x3434 => "nilfs",         // NILFS_SUPER_MAGIC     /* 0x3434 local */
        0x6E736673 => "nsfs",      // NSFS_MAGIC            /* 0x6E736673 local */
        0x5346544E => "ntfs",      // NTFS_SB_MAGIC         /* 0x5346544E local */
        0x9FA1 => "openprom",      // OPENPROM_SUPER_MAGIC  /* 0x9FA1 local */
        0x7461636f => "ocfs2",     // OCFS2_SUPER_MAGIC     /* 0x7461636f remote */
        0xAAD7AAEA => "panfs",     // PANFS_SUPER_MAGIC     /* 0xAAD7AAEA remote */
        0x50495045 => "pipefs",    // PIPEFS_MAGIC          /* 0x50495045 remote */
        0x9FA0 => "proc",          // PROC_SUPER_MAGIC      /* 0x9FA0 local */
        0x6165676C => "pstorefs",  // PSTOREFS_MAGIC        /* 0x6165676C local */
        0x002F => "qnx4",          // QNX4_SUPER_MAGIC      /* 0x002F local */
        0x68191122 => "qnx6",      // QNX6_SUPER_MAGIC      /* 0x68191122 local */
        0x858458F6 => "ramfs",     // RAMFS_MAGIC           /* 0x858458F6 local */
        0x52654973 => "reiserfs",  // REISERFS_SUPER_MAGIC  /* 0x52654973 local */
        0x7275 => "romfs",         // ROMFS_MAGIC           /* 0x7275 local */
        0x67596969 => "rpc_pipefs", // RPC_PIPEFS_SUPER_MAGIC /* 0x67596969 local */
        0x73636673 => "securityfs", // SECURITYFS_SUPER_MAGIC /* 0x73636673 local */
        0xF97CFF8C => "selinux",   // SELINUX_MAGIC         /* 0xF97CFF8C local */
        0x517B => "smb",           // SMB_SUPER_MAGIC       /* 0x517B remote */
        0x534F434B => "sockfs",    // SOCKFS_MAGIC          /* 0x534F434B local */
        0x73717368 => "squashfs",  // SQUASHFS_MAGIC        /* 0x73717368 local */
        0x62656572 => "sysfs",     // SYSFS_MAGIC           /* 0x62656572 local */
        0x012FF7B6 => "sysv2",     // SYSV2_SUPER_MAGIC     /* 0x012FF7B6 local */
        0x012FF7B5 => "sysv4",     // SYSV4_SUPER_MAGIC     /* 0x012FF7B5 local */
        0x01021994 => "tmpfs",     // TMPFS_MAGIC           /* 0x01021994 local */
        0x74726163 => "tracefs",   // TRACEFS_MAGIC         /* 0x74726163 local */
        0x15013346 => "udf",       // UDF_SUPER_MAGIC       /* 0x15013346 local */
        0x00011954 => "ufs",       // UFS_MAGIC             /* 0x00011954 local */
        0x54190100 => "ufs",       // UFS_BYTESWAPPED_SUPER_MAGIC /* 0x54190100 local */
        0x9FA2 => "usbdevfs",      // USBDEVICE_SUPER_MAGIC /* 0x9FA2 local */
        0x01021997 => "v9fs",      // V9FS_MAGIC            /* 0x01021997 local */
        0xBACBACBC => "vmhgfs",    // VMHGFS_SUPER_MAGIC    /* 0xBACBACBC remote */
        0xA501FCF5 => "vxfs",      // VXFS_SUPER_MAGIC      /* 0xA501FCF5 local */
        0x565A4653 => "vzfs",      // VZFS_SUPER_MAGIC      /* 0x565A4653 local */
        0xABBA1974 => "xenfs",     // XENFS_SUPER_MAGIC     /* 0xABBA1974 local */
        0x012FF7B4 => "xenix",     // XENIX_SUPER_MAGIC     /* 0x012FF7B4 local */
        0x58465342 => "xfs",       // XFS_SUPER_MAGIC       /* 0x58465342 local */
        0x012FD16D => "xia",       // _XIAFS_SUPER_MAGIC    /* 0x012FD16D local */
        0x2FC12FC1 => "zfs",       // ZFS_SUPER_MAGIC       /* 0x2FC12FC1 local */
        _ => "",
    }.to_string()
}

fn statfs(path: &str) -> io::Result<libc::statfs> {
    let c_path = CString::new(path).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let mut stat = std::mem::MaybeUninit::<libc::statfs>::uninit();

    let result = unsafe {
        libc::statfs(c_path.as_ptr(), stat.as_mut_ptr())
    };

    if result == 0 {
        Ok(unsafe { stat.assume_init() })
    } else {
        Err(io::Error::last_os_error())
    }
}