import type { ButtonVariant } from '@global/theme';
import * as React from 'react';
import type { IconType } from 'react-icons';
import styled, { css } from 'styled-components';

const MonarchButton = styled.button<{
  $variant: keyof ButtonVariant;
  $disabled: boolean;
  $loading: boolean;
  $fullWidth: boolean;
  $hasLeftIcon: boolean;
  $hasRightIcon: boolean;
}>`
  border-radius: 0.5rem;
  font-size: 1rem;
  font-weight: 700;
  width: ${({ $fullWidth }) => ($fullWidth ? '100%' : 'fit-content')};
  display: flex;
  align-items: center;
  transition: ease-in-out 0.2s;
  max-height: 2.5rem;
  gap: ${({ $variant }) => ($variant === 'menu' ? '1.5rem' : '0.5rem')};
  color: ${({ theme, $variant }) => theme.colors.button[$variant].text};

  padding: ${({ $variant, $hasLeftIcon, $hasRightIcon }) => {
    if ($hasLeftIcon && $hasRightIcon) {
      return '0.5rem';
    }
    if ($hasLeftIcon) {
      return '0.5rem 1rem 0.5rem 0.5rem';
    }
    if ($hasRightIcon) {
      return '0.5rem 0.5rem 0.5rem 1rem';
    }
    return $variant === 'icon' ? '0' : '0.5rem 1rem';
  }};

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
  onClick?: (e: React.MouseEvent<HTMLButtonElement>) => void;
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
      disabled={disabled || loading}
      title={title}
      className={className}
      $hasLeftIcon={leftIcon !== undefined}
      $hasRightIcon={rightIcon !== undefined}
      $variant={variant}
      $disabled={disabled}
      $fullWidth={fullWidth}
      $loading={loading}
    >
      {leftIcon && leftIcon({ size: 24 })}
      {children}
      {rightIcon && rightIcon({ size: 24 })}
    </MonarchButton>
  );
};

export default Button;
