import os
import hashlib
import json
import time
import tkinter as tk
from tkinter import messagebox, filedialog, ttk
import shutil
import subprocess
import tempfile

try:
    from tkinterdnd2 import DND_FILES, TkinterDnD
    DND_AVAILABLE = True
except ImportError:
    DND_AVAILABLE = False
    print("dnd need lib, run: pip install tkinterdnd2")
    TkinterDnD = tk.Tk


class DescGenerator:
    def __init__(self, root):
        self.root = root
        self.root.title("Generator")
        self.root.geometry("680x520")

        self.custom_name = tk.StringVar()
        self.builder = tk.StringVar(value="pipicat613")
        self.version = tk.StringVar(value="1.0.0")
        self.build_version = tk.StringVar(value="1")
        self.desc_filename_var = tk.StringVar()
        self.exe_filename_var = tk.StringVar()

        self.desc_text = None
        self.files_info = []
        self.source_folder = None
        self.create_widgets()

    def create_widgets(self):
        self.drop_frame = tk.Frame(self.root, relief="solid", bg="#e0e0e0", height=60, cursor="hand2")
        self.drop_frame.pack(pady=5, padx=20, fill=tk.X)
        self.drop_frame.pack_propagate(False)

        drop_label = tk.Label(
            self.drop_frame,
            text="点击选择文件夹或将文件夹拖放到此处",
            bg="#e0e0e0",
            font=("Arial", 10)
        )
        drop_label.pack(expand=True, fill=tk.BOTH)
        drop_label.bind("<Button-1>", lambda e: self.select_folder())
        self.drop_frame.bind("<Button-1>", lambda e: self.select_folder())

        if DND_AVAILABLE:
            self.drop_frame.drop_target_register(DND_FILES)
            self.drop_frame.dnd_bind('<<Drop>>', self.on_drop)
            drop_label.drop_target_register(DND_FILES)
            drop_label.dnd_bind('<<Drop>>', self.on_drop)

        input_frame = tk.Frame(self.root)
        input_frame.pack(fill=tk.X, padx=20, pady=5)

        tk.Label(input_frame, text="名称:").grid(row=0, column=0, sticky=tk.W, pady=2)
        tk.Entry(input_frame, textvariable=self.custom_name, width=25).grid(row=0, column=1, sticky=tk.EW, padx=(0,10))
        tk.Label(input_frame, text="构建者:").grid(row=0, column=2, sticky=tk.W, pady=2)
        tk.Entry(input_frame, textvariable=self.builder, width=25).grid(row=0, column=3, sticky=tk.EW)
        input_frame.grid_columnconfigure(1, weight=1)
        input_frame.grid_columnconfigure(3, weight=1)

        tk.Label(input_frame, text="显示版本:").grid(row=1, column=0, sticky=tk.W, pady=2)
        tk.Entry(input_frame, textvariable=self.version, width=25).grid(row=1, column=1, sticky=tk.W, padx=(0,10))
        tk.Label(input_frame, text="构建版本:").grid(row=1, column=2, sticky=tk.W, pady=2)
        tk.Entry(input_frame, textvariable=self.build_version, width=25).grid(row=1, column=3, sticky=tk.W)

        tk.Label(input_frame, text="DESC文件名:").grid(row=2, column=0, sticky=tk.W, pady=2)
        tk.Entry(input_frame, textvariable=self.desc_filename_var, width=25).grid(row=2, column=1, sticky=tk.EW, padx=(0,10))
        tk.Label(input_frame, text="EXE文件名:").grid(row=2, column=2, sticky=tk.W, pady=2)
        tk.Entry(input_frame, textvariable=self.exe_filename_var, width=25).grid(row=2, column=3, sticky=tk.EW)

        tk.Label(self.root, text="文件列表:", anchor="w").pack(fill=tk.X, padx=20, pady=(5,0))
        list_container = tk.Frame(self.root)
        list_container.pack(fill=tk.BOTH, expand=True, padx=20, pady=5)

        self.file_listbox = tk.Listbox(list_container, height=8, width=70, selectmode=tk.SINGLE)
        scrollbar = ttk.Scrollbar(list_container, orient=tk.VERTICAL, command=self.file_listbox.yview)
        self.file_listbox.configure(yscrollcommand=scrollbar.set)
        self.file_listbox.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        scrollbar.pack(side=tk.RIGHT, fill=tk.Y)

        tk.Label(self.root, text="描述:", anchor="w").pack(fill=tk.X, padx=20, pady=(5,0))
        desc_container = tk.Frame(self.root)
        desc_container.pack(fill=tk.BOTH, expand=True, padx=20, pady=5)
        self.desc_text = tk.Text(desc_container, height=5, width=60, wrap=tk.WORD)
        self.desc_text.pack(fill=tk.BOTH, expand=True)

        # 底部按钮
        bottom_frame = tk.Frame(self.root)
        bottom_frame.pack(pady=10)

        ttk.Button(bottom_frame, text="重置", command=self.reset_all).pack(side=tk.LEFT, padx=5)
        ttk.Button(bottom_frame, text="编辑 manifest.json", command=self.edit_manifest).pack(side=tk.LEFT, padx=5)
        ttk.Button(bottom_frame, text="生成DESC", command=self.generate_desc).pack(side=tk.LEFT, padx=5)
        ttk.Button(bottom_frame, text="打包为EXE", command=self.pack_to_sfx).pack(side=tk.LEFT, padx=5)

    # ---------- 重置 ----------
    def reset_all(self):
        self.custom_name.set("")
        self.builder.set("pipicat613")
        self.version.set("1.0.0")
        self.build_version.set("1")
        self.desc_filename_var.set("")
        self.exe_filename_var.set("")
        self.desc_text.delete("1.0", tk.END)
        self.file_listbox.delete(0, tk.END)
        self.files_info = []
        self.source_folder = None

    # ---------- 拖放和选择文件夹 ----------
    def on_drop(self, event):
        paths = self.root.tk.splitlist(event.data)
        if not paths:
            return
        folders = [p for p in paths if os.path.isdir(p)]
        if len(folders) == 1 and len(paths) == 1:
            self.handle_folder(folders[0])
        else:
            messagebox.showwarning("Error", "必须拖入一个文件夹")

    def select_folder(self):
        folder = filedialog.askdirectory()
        if not folder:
            return
        self.handle_folder(folder)

    def handle_folder(self, folder):
        if not os.path.isdir(folder):
            messagebox.showerror("Error", "所选路径不是文件夹")
            return

        self.source_folder = folder
        self._refresh_file_list()   # 扫描并更新列表

        # 检测文件夹中的 DESC 文件
        desc_files = self._find_desc_files(folder)
        if desc_files:
            first_desc = desc_files[0]
            desc_info = self._load_desc_info(first_desc)
            if desc_info is not None:
                answer = messagebox.askyesno("info", f"发现 DESC 文件\n{os.path.basename(first_desc)}\n是否读取这些信息？")
                if answer:
                    self.custom_name.set(desc_info["name"] or os.path.basename(folder))
                    self.builder.set(desc_info["builder"])
                    self.version.set(desc_info["display_version"])
            else:
                messagebox.showwarning("Warn", f"读取 DESC 文件失败：{first_desc}\n可能文件损坏或格式不正确。")

        if not self.files_info:
            messagebox.showwarning("Error", "文件夹中没有可处理的文件")
            return

        # 默认名称使用文件夹名
        self.custom_name.set(os.path.basename(folder))
        self.desc_filename_var.set("")
        self.exe_filename_var.set("")

    # ---------- 刷新文件列表（不改变其他输入） ----------
    def _refresh_file_list(self):
        """重新扫描当前 source_folder，更新 files_info 和列表框"""
        if not self.source_folder or not os.path.isdir(self.source_folder):
            return
        all_files = self._scan_folder(self.source_folder)
        self.files_info = [self._compute_file_info(f, self.source_folder) for f in all_files]
        self.files_info = [info for info in self.files_info if info is not None]

        self.file_listbox.delete(0, tk.END)
        for info in self.files_info:
            self.file_listbox.insert(tk.END, info["relative_path"])

    # ---------- manifest.json 编辑 ----------
    def edit_manifest(self):
        if not self.source_folder:
            messagebox.showwarning("提示", "请先选择或拖入一个文件夹")
            return

        manifest_path = os.path.join(self.source_folder, "manifest.json")
        # 加载现有数据或默认模板
        if os.path.exists(manifest_path):
            try:
                with open(manifest_path, 'r', encoding='utf-8') as f:
                    data = json.load(f)
            except Exception as e:
                messagebox.showerror("错误", f"读取 manifest.json 失败：{e}")
                return
        else:
            data = {"name": "", "description": "", "webpage": ""}

        # 创建编辑窗口
        edit_win = tk.Toplevel(self.root)
        edit_win.title("编辑 manifest.json")
        edit_win.geometry("400x200")
        edit_win.transient(self.root)
        edit_win.grab_set()

        tk.Label(edit_win, text="名称:").grid(row=0, column=0, sticky=tk.W, padx=10, pady=5)
        name_entry = tk.Entry(edit_win, width=40)
        name_entry.grid(row=0, column=1, padx=10, pady=5)
        name_entry.insert(0, data.get("name", ""))

        tk.Label(edit_win, text="描述:").grid(row=1, column=0, sticky=tk.W, padx=10, pady=5)
        desc_entry = tk.Entry(edit_win, width=40)
        desc_entry.grid(row=1, column=1, padx=10, pady=5)
        desc_entry.insert(0, data.get("description", ""))

        tk.Label(edit_win, text="网页:").grid(row=2, column=0, sticky=tk.W, padx=10, pady=5)
        web_entry = tk.Entry(edit_win, width=40)
        web_entry.grid(row=2, column=1, padx=10, pady=5)
        web_entry.insert(0, data.get("webpage", ""))

        def save_manifest():
            new_data = {
                "name": name_entry.get().strip(),
                "description": desc_entry.get().strip(),
                "webpage": web_entry.get().strip()
            }
            try:
                with open(manifest_path, 'w', encoding='utf-8') as f:
                    json.dump(new_data, f, ensure_ascii=False, indent=2)
                messagebox.showinfo("成功", f"manifest.json 已保存到\n{manifest_path}")
                edit_win.destroy()
                # 刷新文件列表（哈希值可能变化）
                self._refresh_file_list()
            except Exception as e:
                messagebox.showerror("错误", f"保存失败：{e}")

        btn_frame = tk.Frame(edit_win)
        btn_frame.grid(row=3, column=0, columnspan=2, pady=15)
        ttk.Button(btn_frame, text="保存", command=save_manifest).pack(side=tk.LEFT, padx=10)
        ttk.Button(btn_frame, text="取消", command=edit_win.destroy).pack(side=tk.LEFT, padx=10)

    # ---------- 扫描相关 ----------
    def _find_desc_files(self, folder):
        desc_files = []
        for root, dirs, files in os.walk(folder):
            dirs[:] = [d for d in dirs if not d.startswith('.')]
            for file in files:
                if file.lower().endswith('.desc'):
                    desc_files.append(os.path.join(root, file))
        return desc_files

    def _load_desc_info(self, desc_path):
        try:
            with open(desc_path, 'r', encoding='utf-8-sig') as f:
                data = json.load(f)
            return {
                "name": data.get("name", ""),
                "builder": data.get("builder", ""),
                "display_version": data.get("display_version", "")
            }
        except Exception as e:
            print(f"读取 DESC 文件失败 {desc_path}: {e}")
            return None

    def _should_ignore(self, path):
        """忽略 .desc 文件（manifest.json 不再忽略）"""
        basename = os.path.basename(path).lower()
        return basename.endswith(".desc")

    def _scan_folder(self, folder):
        file_list = []
        for root, dirs, files in os.walk(folder):
            dirs[:] = [d for d in dirs if not d.startswith('.')]
            for file in files:
                full_path = os.path.join(root, file)
                if not self._should_ignore(full_path):
                    file_list.append(full_path)
        return file_list

    def _compute_file_info(self, filepath, base_folder):
        if not os.path.isfile(filepath):
            return None
        try:
            size = os.path.getsize(filepath)
            sha256_hash = hashlib.sha256()
            with open(filepath, 'rb') as f:
                for chunk in iter(lambda: f.read(4096), b''):
                    sha256_hash.update(chunk)

            rel_path = os.path.relpath(filepath, base_folder)
            return {
                "absolute_path": filepath,
                "relative_path": rel_path,
                "sha256": sha256_hash.hexdigest(),
                "size": size
            }
        except Exception as e:
            print(f"读取文件失败 {filepath}: {e}")
            return None

    # ---------- 构建 DESC 数据 ----------
    def build_desc_data(self):
        if not self.files_info or not self.source_folder:
            messagebox.showwarning("Error", "请先选择或拖入一个文件夹")
            return None

        name = self.custom_name.get().strip()
        if not name:
            messagebox.showwarning("Error", "名称不能为空")
            return None

        builder = self.builder.get().strip()
        if not builder:
            messagebox.showwarning("Error", "构建者不能为空")
            return None

        description = self.desc_text.get("1.0", "end-1c").strip()

        file_list = []
        for info in self.files_info:
            file_list.append({
                "filename": info["relative_path"],
                "sha256": info["sha256"],
                "size": info["size"]
            })

        desc = {
            "name": name,
            "builder": builder,
            "description": description,
            "time": time.time(),
            "display_version": self.version.get().strip(),
            "build_version": self.build_version.get().strip(),
            "file_list": file_list
        }
        return desc

    def get_desc_output_path(self):
        custom = self.desc_filename_var.get().strip()
        if custom:
            if not custom.lower().endswith('.desc'):
                custom += '.desc'
            return os.path.join(self.source_folder, custom)
        else:
            name = self.custom_name.get().strip()
            version = self.version.get().strip()
            return os.path.join(self.source_folder, f"{name}_{version}.desc")

    def get_exe_output_path(self):
        custom = self.exe_filename_var.get().strip()
        if custom:
            if not custom.lower().endswith('.exe'):
                custom += '.exe'
            return os.path.join(os.path.dirname(self.source_folder), custom)
        else:
            folder_name = os.path.basename(self.source_folder)
            version = self.version.get().strip()
            return os.path.join(os.path.dirname(self.source_folder), f"{folder_name}_{version}.exe")

    def write_desc_file(self, desc_data, output_path):
        try:
            with open(output_path, 'w', encoding='utf-8') as f:
                json.dump(desc_data, f, ensure_ascii=False, separators=(',', ':'))
            return True
        except Exception as e:
            messagebox.showerror("Error", f"写入 DESC 文件失败:\n{e}")
            return False

    # ---------- 删除旧 DESC ----------
    def _ask_and_delete_old_desc(self):
        desc_files = self._find_desc_files(self.source_folder)
        if desc_files:
            answer = messagebox.askyesno(
                "删除旧DESC",
                f"发现 {len(desc_files)} 个DESC文件，是否删除它们？\n（删除后只保留新生成的DESC）"
            )
            if answer:
                for f in desc_files:
                    try:
                        os.remove(f)
                    except Exception as e:
                        messagebox.showerror("错误", f"删除文件失败：{f}\n{e}")
                        return False
        return True

    # ---------- 生成 DESC ----------
    def generate_desc(self):
        if not self.source_folder or not self.files_info:
            messagebox.showwarning("Error", "请先选择或拖入一个文件夹")
            return

        if not self._ask_and_delete_old_desc():
            return

        desc_data = self.build_desc_data()
        if desc_data is None:
            return

        output_path = self.get_desc_output_path()
        if self.write_desc_file(desc_data, output_path):
            messagebox.showinfo("OK", f"描述文件已生成:\n{output_path}")

    # ---------- 打包 EXE ----------
    def pack_to_sfx(self):
        if not self.source_folder or not self.files_info:
            messagebox.showwarning("Error", "请先选择或拖入一个文件夹")
            return

        if not self._ask_and_delete_old_desc():
            return

        desc_data = self.build_desc_data()
        if desc_data is None:
            return
        desc_path = self.get_desc_output_path()
        if not self.write_desc_file(desc_data, desc_path):
            return

        seven_zip = shutil.which("7z") or shutil.which("7z.exe")
        if not seven_zip:
            messagebox.showerror("Error", "7z.exe Not Found")
            return

        sfx_module = None
        local_sfx = os.path.join(os.getcwd(), "7zCon.sfx")
        if os.path.isfile(local_sfx):
            sfx_module = local_sfx
        else:
            seven_zip_dir = os.path.dirname(seven_zip)
            candidate = os.path.join(seven_zip_dir, "7zCon.sfx")
            if os.path.isfile(candidate):
                sfx_module = candidate

        if not sfx_module:
            messagebox.showerror("Error", "7zCon.sfx Not Found")
            return

        exe_path = self.get_exe_output_path()
        tmp_dir = tempfile.mkdtemp()
        archive_path = os.path.join(tmp_dir, "archive.7z")

        # 压缩整个文件夹，不再排除 manifest.json
        cmd = [
            seven_zip, "a", "-t7z", archive_path,
            self.source_folder,
        ]
        try:
            subprocess.run(cmd, check=True, capture_output=True)
        except subprocess.CalledProcessError as e:
            messagebox.showerror("Error", f"7-Zip 压缩失败:\n{e.stderr.decode('utf-8', errors='ignore')}")
            shutil.rmtree(tmp_dir, ignore_errors=True)
            return

        try:
            with open(sfx_module, 'rb') as sfx_file, open(archive_path, 'rb') as arch_file, open(exe_path, 'wb') as out_file:
                shutil.copyfileobj(sfx_file, out_file)
                shutil.copyfileobj(arch_file, out_file)
        except Exception as e:
            messagebox.showerror("Error", f"无法生成自解压 EXE:\n{e}")
            shutil.rmtree(tmp_dir, ignore_errors=True)
            return

        shutil.rmtree(tmp_dir, ignore_errors=True)
        messagebox.showinfo("OK", f"自解压 EXE 已生成:\n{exe_path}")


def main():
    if DND_AVAILABLE:
        root = TkinterDnD.Tk()
    else:
        root = tk.Tk()
    app = DescGenerator(root)
    root.mainloop()


if __name__ == "__main__":
    main()