import { AiOutlineSearch } from '@global/icons';
import * as React from 'react';
import styled, { css } from 'styled-components';

import Button from '../button';

const SearchContainer = styled.div`
  display: flex;
  max-height: 2rem;
`;

const StyledInput = styled.input<{
  $hideSearchButton: boolean;
  $fullWidth: boolean;
}>`
  background-color: ${({ theme }) => theme.colors.black};
  border-radius: ${({ $hideSearchButton }) =>
    $hideSearchButton ? '0.5rem' : '0.5rem 0 0 0.5rem'};
  border: none;
  padding: 0.5rem;
  &:focus {
    outline: none;
  }

  ${({ $fullWidth }) =>
    $fullWidth &&
    css`
      width: 100%;
    `}
`;

const SearchButton = styled(Button)`
  border-radius: 0 40% 40% 0;
  border: none;
  padding: 0.2rem 0.5rem;

  &:hover {
    border: none;
    background-color: ${({ theme }) => theme.colors.primary};
    color: ${({ theme }) => theme.colors.secondary};
  }

  &:focus {
    border: none;
  }
`;

type SearchBarProps = {
  value: string;
  onChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
  onSearchClick?: () => void;
  buttonDisabled?: boolean;
  placeholder?: string;
  loading?: boolean;
  hideSearchButton?: boolean;
  autoFocus?: boolean;
  fullWidth?: boolean;
  maxLength?: number;
};

const SearchBar = ({
  value,
  onChange,
  onSearchClick,
  buttonDisabled = false,
  placeholder,
  loading = false,
  hideSearchButton = false,
  autoFocus = false,
  fullWidth = false,
  maxLength,
}: SearchBarProps) => {
  const handleKeyPressed = React.useCallback(
    (e: React.KeyboardEvent<HTMLInputElement>) => {
      if (e.key === 'Enter') {
        onSearchClick?.();
      }
    },
    [onSearchClick],
  );

  return (
    <SearchContainer>
      <StyledInput
        value={value}
        onChange={onChange}
        onKeyDown={handleKeyPressed}
        placeholder={placeholder}
        $hideSearchButton={hideSearchButton}
        data-autofocus={autoFocus}
        $fullWidth={fullWidth}
        maxLength={maxLength}
      />
      {!hideSearchButton && (
        <SearchButton
          type="button"
          variant="secondary"
          onClick={onSearchClick}
          disabled={buttonDisabled}
          loading={loading}
        >
          <AiOutlineSearch size={24} />
        </SearchButton>
      )}
    </SearchContainer>
  );
};

export default SearchBar;
