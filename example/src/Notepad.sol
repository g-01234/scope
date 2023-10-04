pragma solidity ^0.8.0;

contract Notepad {
    function gas() public view returns (address sender, uint256 balance, uint256 gasLeft, uint256 slot0) {
        uint256 slot0;
        assembly {
            slot0 := sload(0)
        }
        return (msg.sender, msg.sender.balance, gasleft(), slot0);
    }
}
