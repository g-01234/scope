pragma solidity ^0.8.0;

import "openzeppelin/token/ERC20/ERC20.sol";

contract SampleERC20 is ERC20 {
    constructor() ERC20("MockToken", "MOCK") {
        _mint(msg.sender, 1e18);
    }

    function mint(address to, uint256 amount) public returns (uint256) {
        _mint(to, amount);
        return balanceOf(to);
    }
}
