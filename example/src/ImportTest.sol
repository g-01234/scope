pragma solidity ^0.8.0;

import "./Imported.sol";

contract Importer is Imported(1) {
    constructor() {}

    function test() public pure {
        assert(1 == 1);
    }
}
