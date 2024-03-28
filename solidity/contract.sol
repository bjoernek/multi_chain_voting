// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

interface IERC20 {
    function totalSupply() external view returns (uint256);

    function balanceOf(address account) external view returns (uint256);

    function transfer(
        address recipient,
        uint256 amount
    ) external returns (bool);

    function allowance(
        address owner,
        address spender
    ) external view returns (uint256);

    function approve(address spender, uint256 amount) external returns (bool);

    function transferFrom(
        address sender,
        address recipient,
        uint256 amount
    ) external returns (bool);

    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(
        address indexed owner,
        address indexed spender,
        uint256 value
    );
}

contract AdminERC20Contract is IERC20 {
    address public admin;
    address public executor;
    mapping(address => uint256) public balances;
    mapping(address => mapping(address => uint256)) public allowances;

    uint256 private _totalSupply;
    address[] private addresses;

    constructor() {
        // Save the deployer's address as the admin
        admin = msg.sender;

        // Assign an initial total supply to the deployer
        _totalSupply = 1000000 * 10 ** 18; // 1 million tokens
        balances[msg.sender] = _totalSupply;
        addresses.push(msg.sender);
    }

    function getAddresses() external view returns (address[] memory) {
        return addresses;
    }

    modifier onlyAdmin() {
        require(msg.sender == admin, "Only admin can call this function");
        _;
    }

    modifier onlyExecutor() {
        require(msg.sender == executor, "Only executor can call this function");
        _;
    }

    function addressExists(address a) internal view returns (bool) {
        for (uint256 i = 0; i < addresses.length; i++) {
            if (addresses[i] == a) {
                return true;
            }
        }
        return false;
    }

    function totalSupply() external view override returns (uint256) {
        return _totalSupply;
    }

    function balanceOf(
        address account
    ) external view override returns (uint256) {
        return balances[account];
    }

    function transfer(
        address recipient,
        uint256 amount
    ) external override returns (bool) {
        require(balances[msg.sender] >= amount, "Insufficient balance");
        balances[msg.sender] -= amount;
        balances[recipient] += amount;
        if (!addressExists(recipient)) {
            addresses.push(recipient); // Add recipient's address to the array if it's not already present
        }
        emit Transfer(msg.sender, recipient, amount);
        return true;
    }

    function allowance(
        address owner,
        address spender
    ) external view override returns (uint256) {
        return allowances[owner][spender];
    }

    function approve(
        address spender,
        uint256 amount
    ) external override returns (bool) {
        allowances[msg.sender][spender] = amount;
        emit Approval(msg.sender, spender, amount);
        return true;
    }

    function transferFrom(
        address sender,
        address recipient,
        uint256 amount
    ) external override returns (bool) {
        require(balances[sender] >= amount, "Insufficient balance");
        require(
            allowances[sender][msg.sender] >= amount,
            "Insufficient allowance"
        );
        balances[sender] -= amount;
        balances[recipient] += amount;
        allowances[sender][msg.sender] -= amount;
        if (!addressExists(recipient)) {
            addresses.push(recipient); // Add recipient's address to the array if it's not already present
        }
        emit Transfer(sender, recipient, amount);
        return true;
    }

    function getAllBalances()
        external
        view
        returns (address[] memory, uint256[] memory)
    {
        uint256 length = 0;
        for (uint256 i = 0; i < addresses.length; i++) {
            if (balances[addresses[i]] > 0) {
                length++;
            }
        }

        address[] memory addrs = new address[](length);
        uint256[] memory bals = new uint256[](length);

        uint256 index = 0;
        for (uint256 i = 0; i < addresses.length; i++) {
            if (balances[addresses[i]] > 0) {
                addrs[index] = addresses[i];
                bals[index] = balances[addresses[i]];
                index++;
            }
        }

        return (addrs, bals);
    }

    enum ProposalState {
        Open,
        Executed
    }

    struct Proposal {
        string description;
        ProposalState state;
    }

    // Example storage for proposals
    mapping(uint256 => Proposal) public proposals;

    // Counter for generating unique proposal IDs
    uint256 private proposalCounter;

    // Function to create a new proposal
    function createProposal(string memory _description) external {
        uint256 newProposalId = proposalCounter++;
        proposals[newProposalId] = Proposal(_description, ProposalState.Open);
    }

    // Function to execute a proposal
    function executeProposal(uint256 _proposalId) external onlyExecutor {
        Proposal storage proposal = proposals[_proposalId];
        require(proposal.state == ProposalState.Open, "Proposal is not open");

        // Perform actions to execute the proposal (example: update state)
        proposal.state = ProposalState.Executed;
    }
}
