from computer_use.mcp_server import TOOLS
from computer_use.protocol import COMMANDS


def test_each_command_has_an_mcp_tool() -> None:
    names = {tool["name"] for tool in TOOLS}
    assert names == {f"computer_{cmd}" for cmd in COMMANDS}
    for tool in TOOLS:
        assert "inputSchema" in tool
        assert tool["inputSchema"]["type"] == "object"
