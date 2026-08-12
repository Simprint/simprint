#!/usr/bin/env python3
"""
构建脚本：编译 Rust 服务并生成包含运行容器所需文件的压缩包。

使用方法:
    python build_docker.py [--release] [--clean] [--no-package]

参数:
    --release: 使用 release 模式编译（默认）
    --clean: 构建前清理旧的二进制文件
    --no-package: 不生成压缩包，只编译和复制文件
"""

import os
import sys
import shutil
import subprocess
import argparse
import zipfile
import tarfile
from pathlib import Path
from datetime import datetime

# 需要编译的服务列表（本仓库仅 simprint-server）
SERVICES = [
    "simprint-server",
]

# 项目根目录
PROJECT_ROOT = Path(__file__).parent.resolve()
TARGET_DIR = PROJECT_ROOT / "target"
RELEASE_DIR = TARGET_DIR / "release"

# 需要打包到压缩包的文件和目录
PACKAGE_FILES = [
    "Dockerfile",
    "docker-compose.yml",
    ".dockerignore",
]

PACKAGE_CONFIG_FILES = [
    ("configs/config.example.toml", "configs/config.toml"),
]

# 需要打包的二进制文件（在构建后复制）
PACKAGE_BINARIES = SERVICES


def print_step(message: str):
    """打印步骤信息"""
    print(f"\n{'='*60}")
    print(f"  {message}")
    print(f"{'='*60}\n")


def check_cargo():
    """检查 cargo 是否可用"""
    try:
        result = subprocess.run(
            ["cargo", "--version"],
            capture_output=True,
            text=True,
            check=True,
        )
        print(f"✓ 找到 Cargo: {result.stdout.strip()}")
        return True
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("✗ 错误: 未找到 Cargo，请确保已安装 Rust 工具链")
        return False


def clean_binaries():
    """清理项目根目录中的旧二进制文件"""
    print_step("清理旧的二进制文件")
    removed = []
    for service in SERVICES:
        binary_path = PROJECT_ROOT / service
        if binary_path.exists():
            binary_path.unlink()
            removed.append(service)
            print(f"  删除: {binary_path}")
    
    if not removed:
        print("  没有需要清理的文件")
    else:
        print(f"  已清理 {len(removed)} 个文件")


def build_service(service: str, release: bool = True) -> bool:
    """编译单个服务"""
    build_mode = "release" if release else "dev"
    print(f"  编译 {service} ({build_mode} 模式)...")
    
    cmd = ["cargo", "build"]
    if release:
        cmd.append("--release")
    cmd.extend(["--bin", service])
    
    try:
        result = subprocess.run(
            cmd,
            cwd=PROJECT_ROOT,
            check=True,
            capture_output=True,
            text=True,
        )
        print(f"  ✓ {service} 编译成功")
        return True
    except subprocess.CalledProcessError as e:
        print(f"  ✗ {service} 编译失败")
        if e.stderr:
            print(f"  错误信息: {e.stderr}")
        return False


def copy_binary(service: str, release: bool = True, dest_dir: Path = None) -> bool:
    """将编译好的二进制文件复制到指定目录"""
    if dest_dir is None:
        dest_dir = PROJECT_ROOT
    
    source_dir = RELEASE_DIR if release else TARGET_DIR / "debug"
    source_path = source_dir / service
    dest_path = dest_dir / service
    
    if not source_path.exists():
        print(f"  ✗ 错误: 源文件不存在: {source_path}")
        return False
    
    try:
        shutil.copy2(source_path, dest_path)
        # 设置可执行权限
        os.chmod(dest_path, 0o755)
        print(f"  ✓ 已复制: {service} -> {dest_path}")
        return True
    except Exception as e:
        print(f"  ✗ 复制失败: {e}")
        return False


def create_package(package_dir: Path, output_format: str = "zip") -> Path:
    """创建压缩包"""
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    package_name = f"simprint-server-docker-{timestamp}"
    
    if output_format == "zip":
        archive_path = PROJECT_ROOT / f"{package_name}.zip"
        print_step(f"创建 ZIP 压缩包: {archive_path.name}")
        
        with zipfile.ZipFile(archive_path, "w", zipfile.ZIP_DEFLATED) as zipf:
            for root, dirs, files in os.walk(package_dir):
                # 跳过隐藏目录
                dirs[:] = [d for d in dirs if not d.startswith(".")]
                
                for file in files:
                    file_path = Path(root) / file
                    arcname = file_path.relative_to(package_dir)
                    zipf.write(file_path, arcname)
                    print(f"  添加: {arcname}")
    
    elif output_format == "tar.gz":
        archive_path = PROJECT_ROOT / f"{package_name}.tar.gz"
        print_step(f"创建 TAR.GZ 压缩包: {archive_path.name}")
        
        with tarfile.open(archive_path, "w:gz") as tar:
            for root, dirs, files in os.walk(package_dir):
                # 跳过隐藏目录
                dirs[:] = [d for d in dirs if not d.startswith(".")]
                
                for file in files:
                    file_path = Path(root) / file
                    arcname = file_path.relative_to(package_dir)
                    tar.add(file_path, arcname=arcname, recursive=False)
                    print(f"  添加: {arcname}")
    
    else:
        raise ValueError(f"不支持的压缩格式: {output_format}")
    
    size_mb = archive_path.stat().st_size / (1024 * 1024)
    print(f"\n✓ 压缩包创建成功: {archive_path.name} ({size_mb:.2f} MB)")
    return archive_path


def prepare_package_directory() -> Path:
    """准备打包目录，复制所有必需的文件"""
    print_step("准备打包目录")
    
    package_dir = PROJECT_ROOT / "docker-package"
    
    # 清理旧的打包目录
    if package_dir.exists():
        shutil.rmtree(package_dir)
    
    package_dir.mkdir(exist_ok=True)
    print(f"  创建打包目录: {package_dir}")
    
    # 复制必需的文件和目录
    copied_count = 0
    for item in PACKAGE_FILES:
        source = PROJECT_ROOT / item
        dest = package_dir / item
        
        if not source.exists():
            print(f"  ⚠ 警告: {item} 不存在，跳过")
            continue
        
        try:
            if source.is_dir():
                shutil.copytree(source, dest, dirs_exist_ok=True)
                print(f"  ✓ 复制目录: {item}")
            else:
                shutil.copy2(source, dest)
                print(f"  ✓ 复制文件: {item}")
            copied_count += 1
        except Exception as e:
            print(f"  ✗ 复制失败 {item}: {e}")

    # 仅复制对外发布需要的示例配置文件
    print(f"\n  复制配置模板:")
    for source_item, dest_item in PACKAGE_CONFIG_FILES:
        source = PROJECT_ROOT / source_item
        dest = package_dir / dest_item

        if not source.exists():
            print(f"  ⚠ 警告: {source_item} 不存在，跳过")
            continue

        try:
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, dest)
            print(f"  ✓ 复制文件: {source_item} -> {dest_item}")
            copied_count += 1
        except Exception as e:
            print(f"  ✗ 复制失败 {source_item}: {e}")
    
    # 复制二进制文件
    print(f"\n  复制二进制文件:")
    for binary in PACKAGE_BINARIES:
        source = PROJECT_ROOT / binary
        if source.exists():
            dest = package_dir / binary
            shutil.copy2(source, dest)
            os.chmod(dest, 0o755)
            print(f"  ✓ 复制: {binary}")
        else:
            print(f"  ✗ 错误: {binary} 不存在")
    
    # 创建 README 文件
    readme_content = f"""# Simprint Server Docker 部署包

本压缩包包含运行 Simprint Server（客户端网关）容器所需的所有文件。

## 包含内容

- 二进制文件: {', '.join(SERVICES)}
- Dockerfile: Docker 镜像构建文件
- docker-compose.yml: Docker Compose 配置文件
- configs/config.toml: 配置模板
- 自动生成的密钥会保存在 Docker 卷中，不随部署包分发

## 使用方法

1. 解压压缩包到目标目录

2. 确保 `configs/config.toml` 中的数据库、Redis、对象存储地址正确

3. 构建 Docker 镜像:
   ```bash
   docker-compose build
   ```

4. 启动服务:
   ```bash
   docker-compose up -d
   ```

5. 查看服务状态:
   ```bash
   docker-compose ps
   ```

服务启动时会自动执行数据库迁移。

6. 查看日志:
   ```bash
   docker-compose logs -f simprint-server
   ```

## 服务端口

- 客户端网关 (simprint-server): 40041

## 注意事项

- 请确保已安装 Docker 和 Docker Compose
- 配置文件中的外部服务地址需要根据实际环境调整
- 首次运行前请检查 configs/ 目录下的配置文件

生成时间: {datetime.now().strftime("%Y-%m-%d %H:%M:%S")}
"""
    
    readme_path = package_dir / "README.md"
    readme_path.write_text(readme_content, encoding="utf-8")
    print(f"  ✓ 创建: README.md")
    
    print(f"\n✓ 打包目录准备完成，共 {copied_count + len(PACKAGE_BINARIES)} 个项目")
    return package_dir


def main():
    parser = argparse.ArgumentParser(
        description="构建 Rust 服务二进制文件供 Docker 使用"
    )
    parser.add_argument(
        "--release",
        action="store_true",
        default=True,
        help="使用 release 模式编译（默认）",
    )
    parser.add_argument(
        "--dev",
        action="store_true",
        help="使用 dev 模式编译",
    )
    parser.add_argument(
        "--clean",
        action="store_true",
        help="构建前清理旧的二进制文件",
    )
    parser.add_argument(
        "--no-package",
        action="store_true",
        help="不生成压缩包，只编译和复制文件到项目根目录",
    )
    parser.add_argument(
        "--format",
        choices=["zip", "tar.gz"],
        default="tar.gz",
        help="压缩包格式 (默认: tar.gz)",
    )
    args = parser.parse_args()
    
    # 确定编译模式
    release_mode = args.release and not args.dev
    
    print_step("Simprint Server Docker 构建脚本")
    print(f"项目目录: {PROJECT_ROOT}")
    print(f"编译模式: {'release' if release_mode else 'dev'}")
    
    # 检查 cargo
    if not check_cargo():
        sys.exit(1)
    
    # 清理旧文件
    if args.clean:
        clean_binaries()
    
    # 编译所有服务
    print_step("编译服务")
    build_success = True
    for service in SERVICES:
        if not build_service(service, release_mode):
            build_success = False
    
    if not build_success:
        print("\n✗ 服务编译失败，请检查错误信息")
        sys.exit(1)
    
    # 复制二进制文件
    print_step("复制二进制文件到项目根目录")
    copy_success = True
    for service in SERVICES:
        if not copy_binary(service, release_mode):
            copy_success = False
    
    if not copy_success:
        print("\n✗ 文件复制失败")
        sys.exit(1)
    
    # 验证文件
    print_step("验证构建结果")
    all_exist = True
    for service in SERVICES:
        binary_path = PROJECT_ROOT / service
        if binary_path.exists():
            size = binary_path.stat().st_size / (1024 * 1024)  # MB
            print(f"  ✓ {service}: {size:.2f} MB")
        else:
            print(f"  ✗ {service}: 文件不存在")
            all_exist = False
    
    if not all_exist:
        print("\n✗ 构建验证失败")
        sys.exit(1)
    
    # 生成压缩包
    if not args.no_package:
        package_dir = prepare_package_directory()
        archive_path = create_package(package_dir, args.format)
        
        # 清理打包目录
        print_step("清理临时文件")
        shutil.rmtree(package_dir)
        print("  ✓ 已清理临时打包目录")
        
        print_step("构建完成！")
        print(f"\n✓ 压缩包已生成: {archive_path.name}")
        print(f"  位置: {archive_path}")
        print("\n现在可以：")
        print("  1. 将压缩包传输到目标服务器")
        print("  2. 解压后运行: docker-compose up -d")
    else:
        print_step("构建完成！")
        print("\n现在可以使用以下命令构建 Docker 镜像：")
        print("  docker-compose build")
        print("\n或者启动服务：")
        print("  docker-compose up -d")


if __name__ == "__main__":
    main()

