// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract StringStorage {
    string[10] lastTenStrings;
    uint8 currentIndex;

    function storeString(string memory _str) public {
        currentIndex = (currentIndex + 1) % 10;
        lastTenStrings[currentIndex] = _str;
    }

    function getLastTenStrings() public view returns (string[10] memory) {
        string[10] memory recentStrings;
        for (uint8 i = 0; i < 10; i++) {
            recentStrings[i] = lastTenStrings[(10 + currentIndex - i) % 10];
        }
        return recentStrings;
    }
}
