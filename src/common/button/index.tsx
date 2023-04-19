import * as React from 'react';
import styled from 'styled-components';
import { type IconType } from 'react-icons';
import { type ButtonVariant } from '../../global/theme';

const MonarchButton = styled.button<{
  $variant: keyof ButtonVariant;
  $disabled: boolean;
  $width: string;
}>`
  border-radius: 0.5rem;
  font-size: 1rem;
  font-weight: 700;
  padding: 0.5rem 1rem;
  width: ${({ $width }) => $width};
  transition: ease 0.2s;
  display: flex;
  align-items: center;
  gap: 1.5rem;
  max-height: 2.5rem;

  color: ${({ theme, $variant }) => theme.colors.button[$variant].text};

  background-color: ${({ theme, $variant }) =>
    theme.colors.button[$variant].background};

  border: 0.2rem solid
    ${({ theme, $variant }) => theme.colors.button[$variant].border};

  // Hover state
  &:hover {
    color: ${({ theme, $variant }) => theme.colors.button[$variant].hoverText};

    background-color: ${({ theme, $variant }) =>
      theme.colors.button[$variant].hoverBackground};

    border: 0.2rem solid
      ${({ theme, $variant }) => theme.colors.button[$variant].hoverBorder};

    cursor: ${({ $disabled }) => ($disabled ? 'not-allowed' : 'pointer')};
  }

  //Focus state
  &:focus {
    color: ${({ theme, $variant }) => theme.colors.button[$variant].focusText};

    background-color: ${({ theme, $variant }) =>
      theme.colors.button[$variant].focusBackground};

    border: 0.2rem solid
      ${({ theme, $variant }) => theme.colors.button[$variant].focusBorder};
  }
`;

type ButtonProps = {
  type: 'button' | 'submit' | 'reset';
  variant: keyof ButtonVariant;
  onClick: () => void;
  disabled?: boolean;
  children: React.ReactNode;
  leftIcon?: IconType;
  rightIcon?: IconType;
  width?: string;
};

const Button = ({
  type,
  variant,
  onClick,
  disabled = false,
  children,
  leftIcon,
  rightIcon,
  width = 'fit-content',
}: ButtonProps) => {
  return (
    <MonarchButton
      type={type}
      onClick={onClick}
      $variant={variant}
      $disabled={disabled}
      $width={width}
      disabled={disabled}
    >
      {leftIcon && leftIcon({ size: 24 })}
      {children}
      {rightIcon && rightIcon({ size: 24 })}
    </MonarchButton>
  );
};

export default Button;