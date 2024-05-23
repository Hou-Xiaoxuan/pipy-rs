use cmake::Config;

fn main() {
    // TODO Randomly compilable now.
    let mut config = Config::new("path/to/cmake/project");

    // 根据需要设置 CMake 选项
    config.define("PIPY_SHARED", "On");

    // 配置构建并生成项目
    let dst = config.build();

    // 将生成的库路径添加到 rustc 的链接路径中
    println!("cargo:rustc-link-search=native={}", dst.display());

    // 根据生成的库名称链接相应的库
    println!("cargo:rustc-link-lib=static=pipy");
}
