import React from 'react';

const Spinner: React.FC = () => {
  return (
    <svg width="50" height="50" viewBox="0 0 50 50">
      <circle
        cx="25"
        cy="25"
        r="20"
        stroke="#09f"
        strokeWidth="4"
        fill="none"
        strokeDasharray="31.415, 31.415"
        strokeLinecap="round"
      >
        <animateTransform
          attributeName="transform"
          type="rotate"
          repeatCount="indefinite"
          from="0 25 25"
          to="360 25 25"
          dur="1s"
        />
      </circle>
    </svg>
  );
};

export default Spinner;
