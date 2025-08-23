#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Target {
    I386,
    Amd64,
    Arm64,
}

impl Target {
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "i386" | "i686" | "x86" => Some(Target::I386),
            "amd64" | "x86_64" | "x64" => Some(Target::Amd64),
            "arm64" | "aarch64" => Some(Target::Arm64),
            "native" => Some(Self::native()),
            _ => None,
        }
    }

    pub fn native() -> Self {
        #[cfg(target_arch = "x86")]
        return Target::I386;

        #[cfg(target_arch = "x86_64")]
        return Target::Amd64;

        #[cfg(target_arch = "aarch64")]
        return Target::Arm64;

        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
        return Target::Amd64; // Default fallback
    }

    pub fn pointer_size(&self) -> usize {
        match self {
            Target::I386 => 4,
            Target::Amd64 => 8,
            Target::Arm64 => 8,
        }
    }

    #[allow(dead_code)]
    pub fn alignment(&self) -> usize {
        match self {
            Target::I386 => 4,
            Target::Amd64 => 8,
            Target::Arm64 => 8,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Target::I386 => "i386",
            Target::Amd64 => "amd64",
            Target::Arm64 => "arm64",
        }
    }

    #[allow(dead_code)]
    pub fn triple(&self) -> &'static str {
        match self {
            Target::I386 => "i386-unknown-linux-gnu",
            Target::Amd64 => "x86_64-unknown-linux-gnu",
            Target::Arm64 => "aarch64-unknown-linux-gnu",
        }
    }

    #[allow(dead_code)]
    pub fn assembler(&self) -> &'static str {
        match self {
            Target::I386 => "as --32",
            Target::Amd64 => "as --64",
            Target::Arm64 => "aarch64-linux-gnu-as",
        }
    }

    #[allow(dead_code)]
    pub fn linker(&self) -> &'static str {
        match self {
            Target::I386 => "ld -m elf_i386",
            Target::Amd64 => "ld -m elf_x86_64",
            Target::Arm64 => "aarch64-linux-gnu-ld",
        }
    }

    #[allow(dead_code)]
    pub fn object_format(&self) -> &'static str {
        match self {
            Target::I386 => "elf32",
            Target::Amd64 => "elf64",
            Target::Arm64 => "elf64",
        }
    }

    #[allow(dead_code)]
    pub fn calling_convention(&self) -> CallingConvention {
        match self {
            Target::I386 => CallingConvention::Cdecl,
            Target::Amd64 => CallingConvention::SystemV,
            Target::Arm64 => CallingConvention::Aapcs64,
        }
    }

    #[allow(dead_code)]
    pub fn register_names(&self) -> RegisterSet {
        match self {
            Target::I386 => RegisterSet::X86_32,
            Target::Amd64 => RegisterSet::X86_64,
            Target::Arm64 => RegisterSet::Aarch64,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum CallingConvention {
    Cdecl,   // x86-32
    SystemV, // x86-64
    Aapcs64, // ARM64
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum RegisterSet {
    X86_32,
    X86_64,
    Aarch64,
}

#[allow(dead_code)]
impl RegisterSet {
    pub fn general_purpose_registers(&self) -> &'static [&'static str] {
        match self {
            RegisterSet::X86_32 => &["eax", "ebx", "ecx", "edx", "esi", "edi"],
            RegisterSet::X86_64 => &[
                "rax", "rbx", "rcx", "rdx", "rsi", "rdi", "r8", "r9", "r10", "r11", "r12", "r13",
                "r14", "r15",
            ],
            RegisterSet::Aarch64 => &[
                "x0", "x1", "x2", "x3", "x4", "x5", "x6", "x7", "x8", "x9", "x10", "x11", "x12",
                "x13", "x14", "x15", "x16", "x17", "x18", "x19", "x20", "x21", "x22", "x23", "x24",
                "x25", "x26", "x27", "x28",
            ],
        }
    }

    pub fn parameter_registers(&self) -> &'static [&'static str] {
        match self {
            RegisterSet::X86_32 => &[], // Parameters passed on stack
            RegisterSet::X86_64 => &["rdi", "rsi", "rdx", "rcx", "r8", "r9"],
            RegisterSet::Aarch64 => &["x0", "x1", "x2", "x3", "x4", "x5", "x6", "x7"],
        }
    }

    pub fn return_register(&self) -> &'static str {
        match self {
            RegisterSet::X86_32 => "eax",
            RegisterSet::X86_64 => "rax",
            RegisterSet::Aarch64 => "x0",
        }
    }

    pub fn stack_pointer(&self) -> &'static str {
        match self {
            RegisterSet::X86_32 => "esp",
            RegisterSet::X86_64 => "rsp",
            RegisterSet::Aarch64 => "sp",
        }
    }

    pub fn frame_pointer(&self) -> &'static str {
        match self {
            RegisterSet::X86_32 => "ebp",
            RegisterSet::X86_64 => "rbp",
            RegisterSet::Aarch64 => "x29",
        }
    }
}

#[allow(dead_code)]
pub struct TargetInfo {
    pub target: Target,
    pub endianness: Endianness,
    pub word_size: usize,
    pub max_align: usize,
    pub supports_pic: bool,
    pub supports_pie: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum Endianness {
    Little,
    Big,
}

#[allow(dead_code)]
impl TargetInfo {
    pub fn new(target: Target) -> Self {
        let (word_size, max_align) = match target {
            Target::I386 => (4, 4),
            Target::Amd64 => (8, 8),
            Target::Arm64 => (8, 16),
        };

        Self {
            target,
            endianness: Endianness::Little, // All supported targets are little-endian
            word_size,
            max_align,
            supports_pic: true,
            supports_pie: true,
        }
    }

    pub fn size_of_type(&self, type_name: &str) -> Option<usize> {
        match type_name {
            "char" | "signed char" | "unsigned char" => Some(1),
            "short" | "unsigned short" => Some(2),
            "int" | "unsigned int" => Some(4),
            "long" | "unsigned long" => Some(self.word_size),
            "long long" | "unsigned long long" => Some(8),
            "float" => Some(4),
            "double" => Some(8),
            "long double" => match self.target {
                Target::I386 => Some(12),
                Target::Amd64 => Some(16),
                Target::Arm64 => Some(16),
            },
            "void*" | "size_t" | "ptrdiff_t" => Some(self.word_size),
            _ => None,
        }
    }

    pub fn align_of_type(&self, type_name: &str) -> Option<usize> {
        match type_name {
            "char" | "signed char" | "unsigned char" => Some(1),
            "short" | "unsigned short" => Some(2),
            "int" | "unsigned int" => Some(4),
            "long" | "unsigned long" => Some(self.word_size),
            "long long" | "unsigned long long" => Some(8),
            "float" => Some(4),
            "double" => Some(8),
            "long double" => match self.target {
                Target::I386 => Some(4),
                Target::Amd64 => Some(16),
                Target::Arm64 => Some(16),
            },
            "void*" | "size_t" | "ptrdiff_t" => Some(self.word_size),
            _ => None,
        }
    }
}
