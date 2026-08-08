# tests/test_single_word_python.py
# ============================================================================
# AST AUDIT FOR SINGLE-WORD IDENTIFIERS (RULE 3.2)
# Challenger Agent: challenger_m1_i3_1
# ============================================================================

import ast
import glob
import os
import unittest

class TestSingleWordPython(unittest.TestCase):
    """Kiểm tra AST cho quy tắc Single-Word English Identifiers."""

    def test_single_word_identifiers_m1_scripts(self):
        m1_script_files = [
            "scripts/gpu_mine.py",
            "scripts/hub.py",
            "scripts/deploy_dataset.py",
            "scripts/mine.py",
            "scripts/train.py",
            "scripts/llm_server.py"
        ]
        
        allowed_exceptions = {
            "hf_hub_download", "load_dataset", "push_to_hub", "save_method",
            "per_device_train_batch_size", "gradient_accumulation_steps", "max_seq_length",
            "learning_rate", "output_dir", "repo_id", "repo_type", "force_download",
            "path_or_fileobj", "path_in_repo", "is_available", "cuda_is_available",
            "__name__", "__main__", "__file__", "__doc__", "__cause__",
            "__init__", "__str__", "__repr__",
            "do_GET", "do_POST", "do_OPTIONS"
        }

        violations_found = {}

        for filepath in m1_script_files:
            if not os.path.exists(filepath):
                continue
            with open(filepath, "r", encoding="utf-8") as f:
                content = f.read()
            tree = ast.parse(content, filename=filepath)
            
            file_violations = []
            for node in ast.walk(tree):
                if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                    if "_" in node.name and node.name not in allowed_exceptions:
                        file_violations.append((node.lineno, "function", node.name))
                elif isinstance(node, ast.Name) and isinstance(node.ctx, ast.Store):
                    if "_" in node.id and not node.id.startswith("__") and node.id not in allowed_exceptions:
                        file_violations.append((node.lineno, "variable", node.id))
                        
            if file_violations:
                violations_found[filepath] = file_violations

        if violations_found:
            msg = f"Phát hiện vi phạm định danh từ ghép multi-word (snake_case) trong các tệp M1:\n"
            for path, viols in violations_found.items():
                msg += f"  File {path}:\n"
                for line, kind, name in viols:
                    msg += f"    Line {line}: {kind} '{name}'\n"
            self.fail(msg)
        else:
            print("✅ 100% Core M1 Python scripts (gpu_mine, hub, deploy_dataset, mine, train) follow Single-Word English Identifiers!")

if __name__ == "__main__":
    unittest.main()
