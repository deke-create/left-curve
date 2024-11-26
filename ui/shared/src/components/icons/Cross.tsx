import type React from "react";

export const CrossIcon: React.FC<React.SVGAttributes<HTMLOrSVGElement>> = ({ ...props }) => {
  return (
    <svg
      width="10"
      height="10"
      viewBox="0 0 10 10"
      fill="currentColor"
      xmlns="http://www.w3.org/2000/svg"
      {...props}
    >
      <path
        fillRule="evenodd"
        clipRule="evenodd"
        d="M7.98536 9.24755C8.33404 9.59628 8.89936 9.59628 9.24804 9.24755C9.59678 8.89888 9.59678 8.33355 9.24804 7.98487L6.26331 5.00014L9.24804 2.01541C9.59678 1.66674 9.59678 1.10141 9.24804 0.752725C8.89936 0.40404 8.33404 0.40404 7.98536 0.752725L5.00063 3.73745L2.0159 0.752725C1.66721 0.40404 1.10188 0.40404 0.753213 0.752725C0.404528 1.10141 0.404528 1.66673 0.753213 2.01541L3.73794 5.00014L0.753213 7.98487C0.404528 8.33355 0.404528 8.89888 0.753213 9.24757C1.10188 9.59628 1.66721 9.59628 2.0159 9.24757L5.00063 6.26284L7.98536 9.24755Z"
      />
    </svg>
  );
};
