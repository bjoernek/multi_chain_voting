import React from 'react';
import icLogo from '../../public/ic.svg'; // Ensure the path is correct
import ethLogo from '../../public/eth.svg'; // Ensure the path is correct

// Define a TypeScript interface for props if using TypeScript
interface SpinnerProps {
  item: string; // The item being fetched or synchronized
}

const Spinner: React.FC<SpinnerProps> = ({ item }) => {
  return (
    <div className="spinner-container" style={{ position: 'relative', width: '150px', height: '150px', display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center' }}>
      <svg width="150" height="150" viewBox="0 0 100 100">
        <circle
          cx="50"
          cy="50"
          r="45"
          stroke="#ccc"
          strokeWidth="5"
          fill="none"
        />
        <image href={icLogo} x="40" y="10" height="30px" width="30px" transform="translate(5,35)">
          <animateTransform
            attributeName="transform"
            type="rotate"
            repeatCount="indefinite"
            from="0 50 50"
            to="360 50 50"
            dur="2s"
          />
        </image>
        <image href={ethLogo} x="40" y="60" height="40px" width="40px" transform="translate(0,30)">
          <animateTransform
            attributeName="transform"
            type="rotate"
            repeatCount="indefinite"
            from="360 50 50"
            to="0 50 50"
            dur="2s"
          />
        </image>
      </svg>
      <div className="text-center mt-3" style={{ fontSize: '14px' }}>
        Fetching {item} from Ethereum
      </div>
    </div>
  );
};

export default Spinner;
