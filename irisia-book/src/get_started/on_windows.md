# Windows环境

## 安装Rust

请跟随[官网指引](https://www.rust-lang.org/tools/install)，安装Rust。

## 将irisia引入您的项目

创建一个新项目，并进入该目录

```shell
> cargo new my-first-app
> cd my-first-app
```

打开`Cargo.toml`文件，并在项目的`[dependencies]`下添加依赖。由于该库尚未正式发布，因此仅能通过git链接进行安装。

```toml
irisia = { git = "https://github.com/Fancyflame/irisia-rs.git" }
```

然后等待安装完成。

## 如果安装过程中`skia-safe`编译错误

有时因为各种原因，`skia-safe`无法正常编译。在本文中，推荐使用预编译包，这样您无需安装skia编译环境并等待编译。如无法下载预编译包，请检查网络连接是否正常。若仍无法下载，请采取手动下载方式，如下文。

1. 进入[`skia-binaries`](https://github.com/rust-skia/skia-binaries/releases)发布页
2. 下载最新预编译包，格式为`skia-binaries-<编号>-<架构>-pc-windows-msvc-textlayout.tar.gz`,如`skia-binaries-4f106aa048fa92fce6ce-x86_64-pc-windows-msvc-textlayout.tar.gz`
3. 将压缩包复制到合适的文件夹，并选择下列任意一种方法
   - （推荐）在系统搜索栏中搜索`PATH`，选择编辑系统环境变量，新建环境变量`SKIA_BINARIES_URL`，值为`file://<目录>\skia-binaries-{key}.tar.gz`，例如`file://C:\path\to\directory\skia-binaries-{key}.tar.gz`。注意，`{key}`要保留源文本
   - 启动文件服务器[^1]，将上面步骤的值改为`<服务器地址>/skia-binaries-{key}.tar.gz`，例如`http://127.0.0.1:8000/skia-binaries-{key}.tar.gz`
4. 重启您的控制台，或设置同样的环境变量，使得您的控制台能够识别您新设置的`SKIA_BINARIES_URL`环境变量
5. 重新编译一次

## 如果出现skia链接错误

原issue [rust-skia#660](https://github.com/rust-skia/rust-skia/issues/660)
主要原因是构建工具MSBuild版本过低，比如您使用的是MSBuild 2019。您可以参考以下更新方案：
1. 访问<https://visualstudio.microsoft.com/zh-hans/visual-cpp-build-tools/>获得MSBuild安装程序
2. 启动安装程序，安装最新Visual Studio构建工具
3. 点击“修改”，打开“单个组件”
4. 勾选最新MSVC生成工具，例如`MSVC v143 - VS 2022 C++ x64/x86 生成工具(最新)`
5. 勾选最新Windows SDK，例如`Windows 11 SDK (10.0.22621.0)`
6. 点击“修改”，等待安装完成
7. 重新编译一次

[^1]: 推荐选用nodejs或python文件服务器。如果您选用的是nodejs服务器，输入`http-server C:\path\to\directory -p 8000`来启动（需安装[`http-server`](https://www.npmjs.com/package/http-server)）。如果您选用的是python服务器，输入`python -m http.server -d C:\path\to\directory -p 8000`来启动。
