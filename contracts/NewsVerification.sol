// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract NewsVerification {
    struct Verification {
        bytes32 articleHash;
        string verificationData;
        uint256 timestamp;
        address verifier;
        bool isValid;
    }

    mapping(bytes32 => Verification) public verifications;
    mapping(address => bool) public authorizedVerifiers;

    event VerificationCreated(
        bytes32 indexed articleHash,
        address indexed verifier,
        uint256 timestamp
    );
    event VerificationUpdated(
        bytes32 indexed articleHash,
        address indexed verifier,
        uint256 timestamp
    );
    event VerifierAuthorized(address indexed verifier);
    event VerifierRevoked(address indexed verifier);

    modifier onlyAuthorized() {
        require(authorizedVerifiers[msg.sender], "Not authorized");
        _;
    }

    constructor() {
        authorizedVerifiers[msg.sender] = true;
    }

    function createVerificationProof(
        bytes32 articleHash,
        string memory verificationData
    ) external onlyAuthorized returns (bool) {
        require(verifications[articleHash].timestamp == 0, "Verification already exists");

        verifications[articleHash] = Verification({
            articleHash: articleHash,
            verificationData: verificationData,
            timestamp: block.timestamp,
            verifier: msg.sender,
            isValid: true
        });

        emit VerificationCreated(articleHash, msg.sender, block.timestamp);
        return true;
    }

    function updateVerification(
        bytes32 articleHash,
        string memory verificationData
    ) external onlyAuthorized returns (bool) {
        require(verifications[articleHash].timestamp != 0, "Verification does not exist");

        verifications[articleHash].verificationData = verificationData;
        verifications[articleHash].timestamp = block.timestamp;
        verifications[articleHash].verifier = msg.sender;

        emit VerificationUpdated(articleHash, msg.sender, block.timestamp);
        return true;
    }

    function getVerificationState(bytes32 articleHash)
        external
        view
        returns (string memory)
    {
        require(verifications[articleHash].timestamp != 0, "Verification does not exist");
        return verifications[articleHash].verificationData;
    }

    function verifyProof(bytes32 articleHash) external view returns (bool) {
        return verifications[articleHash].isValid;
    }

    function authorizeVerifier(address verifier) external onlyAuthorized {
        authorizedVerifiers[verifier] = true;
        emit VerifierAuthorized(verifier);
    }

    function revokeVerifier(address verifier) external onlyAuthorized {
        authorizedVerifiers[verifier] = false;
        emit VerifierRevoked(verifier);
    }
} 