import React from 'react';
import icLogo from '../../public/ic.svg';
import ethLogo from '../../public/eth.svg';

interface SpinnerProps {
  item: string;
}

const Spinner: React.FC<SpinnerProps> = ({ item }) => {
  return (
    <div className="spinner-container" style={{ position: 'relative', width: '200px', height: '200px', display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center' }}>
      <svg width="200" height="200" viewBox="0 0 100 100">
        <circle
          cx="50"
          cy="50"
          r="45"
          stroke="#ccc"
          strokeWidth="7"
          fill="none"
        />
        <image href={icLogo} x="10" y="30" height="40px" width="40px" >
          <animateTransform
            attributeName="transform"
            type="rotate"
            repeatCount="indefinite"
            from="0 50 50"
            to="360 50 50"
            dur="2s"
          />
        </image>
        <image href={ethLogo} x="60" y="30" height="40px" width="40px"> 
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



      <div className="text-center text-white mt-3" style={{ fontSize: '18px' }}>
        Fetching {item} from Ethereum Sepolia
      </div>
    </div>
  );
};

export default Spinner;
