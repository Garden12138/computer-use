from pathlib import Path

from computer_use.client import HelperClient
from computer_use.runtime import Computer


def test_fake_helper_doctor_shape(tmp_path: Path) -> None:
    script = tmp_path / "fake-helper"
    script.write_text(
        "#!/usr/bin/env python3\n"
        "import json,sys\n"
        "req=json.loads(sys.stdin.readline())\n"
        "print(json.dumps({'id':req['id'],'ok':True,'data':{"
        "'accessibility':True,'screen_recording':True,"
        "'bundle_id':'dev.computeruse.helper','hints':[],'ready':True}}))\n",
        encoding="utf-8",
    )
    script.chmod(0o755)
    computer = Computer(pacing="off", client=HelperClient(script))
    result = computer.doctor()
    assert result["ok"] is True
    assert result["data"]["ready"] is True
    assert result["data"]["bundle_id"] == "dev.computeruse.helper"
    assert "helper" in result["data"]
