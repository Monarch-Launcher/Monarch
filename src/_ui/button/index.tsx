import * as React from 'react';
import styled, { css } from 'styled-components';
import { type IconType } from 'react-icons';
import { type ButtonVariant } from '../../global/theme';

const MonarchButton = styled.button<{
  $variant: keyof ButtonVariant;
  $disabled: boolean;
  $loading: boolean;
  $fullWidth: boolean;
}>`
  border-radius: 0.5rem;
  font-size: 1rem;
  font-weight: 700;
  padding: ${({ $variant }) => ($variant === 'icon' ? '0' : '0.5rem 1rem')};
  width: ${({ $fullWidth }) => ($fullWidth ? '100%' : 'fit-content')};
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

    ${({ $disabled, $loading }) =>
      css`
        cursor: pointer;
        ${$disabled &&
        css`
          cursor: not-allowed;
        `}
        ${$loading &&
        css`
          cursor: progress;
        `}
      `}
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
  loading?: boolean;
  children: React.ReactNode;
  leftIcon?: IconType;
  rightIcon?: IconType;
  fullWidth?: boolean;
  className?: string;
  title?: string;
};

const Button = ({
  type,
  variant,
  onClick,
  disabled = false,
  loading = false,
  children,
  leftIcon,
  rightIcon,
  fullWidth = false,
  title,
  className,
}: ButtonProps) => {
  return (
    <MonarchButton
      type={type}
      onClick={onClick}
      $variant={variant}
      $disabled={disabled}
      $fullWidth={fullWidth}
      $loading={loading}
      disabled={disabled || loading}
      className={className}
      title={title}
    >
      {leftIcon && leftIcon({ size: 24 })}
      {children}
      {rightIcon && rightIcon({ size: 24 })}
    </MonarchButton>
  );
};

export default Button;
