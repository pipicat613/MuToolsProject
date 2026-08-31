import os
import sys
import shutil
import subprocess
import tempfile

def check_dependencies():
    if shutil.which('7z.exe') is None:
        print('7z.exe Not Found')
        sys.exit(1)

    if not os.path.isfile('7zS2.sfx'):
        print('7zS2.sfx Not Found')
        sys.exit(1)

    rcedit_names = ['rcedit.exe', 'rcedit-x64.exe']
    rcedit_path = None
    for name in rcedit_names:
        if os.path.isfile(name):
            rcedit_path = name
            break
    if rcedit_path is None:
        for name in rcedit_names:
            found = shutil.which(name)
            if found:
                rcedit_path = found
                break
    if rcedit_path is None:
        print('rcedit.exe or rcedit-x64.exe Not Found')
        sys.exit(1)

    if not os.path.isfile(os.path.join('dist', 'mutools.exe')):
        print('./dist/mutools.exe Not Found')
        sys.exit(1)
    if not os.path.isfile('icon.ico'):
        print('icon.ico Not Found')
        sys.exit(1)

    return rcedit_path

def main():
    script_dir = os.path.dirname(os.path.abspath(__file__))
    os.chdir(script_dir)

    rcedit_path = check_dependencies()

    output_exe = os.path.join('dist', 'MuTools_portable.exe')
    # 在 ./dist 目录下创建临时目录
    temp_dir = tempfile.mkdtemp(dir=os.path.join(script_dir, 'dist'))

    try:
        print(f'Temp File: {temp_dir}')

        # 配置文件和归档文件直接放在临时目录下
        config_path = os.path.join(temp_dir, 'config.txt')
        with open(config_path, 'w', encoding='utf-8') as f:
            f.write(';!@Install@!UTF-8!\n')
            f.write('RunProgram="mutools.exe"\n')
            f.write(';!@InstallEnd@!\n')
        print('config.txt OK')

        staging_dir = os.path.join(temp_dir, 'archive')
        os.makedirs(staging_dir)
        shutil.copy2('./dist/mutools.exe', staging_dir)

        archive_path = os.path.join(temp_dir, 'archive.7z')
        cmd_7z = [
            '7z', 'a', '-t7z', '-mx=9',
            archive_path,
            os.path.join(staging_dir, '*'),
            '-r'
        ]
        result = subprocess.run(cmd_7z, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        if result.returncode != 0:
            print('To 7z error:', result.stderr.decode('gbk', errors='ignore'))
            sys.exit(1)
        print('To 7z OK')

        modified_sfx_path = os.path.join(temp_dir, 'sfx_modified.sfx')
        shutil.copy2('7zS2.sfx', modified_sfx_path)

        cmd_icon = [rcedit_path, modified_sfx_path, '--set-icon', 'icon.ico']
        result = subprocess.run(cmd_icon, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        if result.returncode != 0:
            print('Change icon error:', result.stderr.decode('gbk', errors='ignore'))
            sys.exit(1)
        print('Change icon OK')

        # 合并生成最终可执行文件
        with open(output_exe, 'wb') as out_f:
            with open(modified_sfx_path, 'rb') as sfx_f:
                shutil.copyfileobj(sfx_f, out_f)
            with open(config_path, 'rb') as cfg_f:
                shutil.copyfileobj(cfg_f, out_f)
            with open(archive_path, 'rb') as arc_f:
                shutil.copyfileobj(arc_f, out_f)

    finally:
        shutil.rmtree(temp_dir, ignore_errors=True)
        print('Temp file clean OK')

    print(f'output: {output_exe}')

if __name__ == '__main__':
    try:
        main()
    except Exception as e:
        print(e)
    input("done.")