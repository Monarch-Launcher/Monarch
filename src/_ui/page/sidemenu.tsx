import Logo from '@assets/logo.svg';
import {
  AiOutlineSearch,
  BiHomeAlt,
  BiTestTube,
  HiOutlineCog,
  HiOutlineSquares2X2,
} from '@global/icons';
import * as React from 'react';
import type { IconType } from 'react-icons';
import { useLocation, useNavigate } from 'react-router-dom';
import styled, { AnyStyledComponent } from 'styled-components';

import Button from '../button';

const Container = styled.div`
  background-color: ${({ theme }) => theme.colors.secondary};
  width: 12rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 0.5rem;
`;

const Header = styled.div``;

const NavContainer = styled.div`
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  height: 100%;
`;

const TabContainer = styled.div`
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
`;

const LinkWrapper = styled.div<{ $isActive: boolean }>`
  border-radius: 0.5rem;

  background-color: ${({ theme, $isActive }) =>
    $isActive
      ? theme.colors.button.transparent.active
      : theme.colors.button.transparent.background};
`;

// TODO: Proper svg setup for styled components
const StyledLogo = styled(Logo as AnyStyledComponent)`
  margin: 0.5rem 0.5rem 0;
  .st0 {
    fill: ${({ theme }) => theme.colors.primary};
  }
`;

type Tab = {
  id: number;
  path: string;
  title: string;
  leftIcon: IconType;
};

const SideMenu = () => {
  const navigate = useNavigate();
  const { pathname } = useLocation();

  const navigateTo = React.useCallback(
    (path: string) => {
      navigate(path);
    },
    [navigate],
  );

  const tabs = React.useMemo<Tab[]>(() => {
    return [
      {
        id: 1,
        path: '/',
        title: 'Home',
        leftIcon: BiHomeAlt,
      },
      {
        id: 2,
        path: '/library',
        title: 'Library',
        leftIcon: HiOutlineSquares2X2,
      },
      {
        id: 3,
        path: '/search',
        title: 'Search',
        leftIcon: AiOutlineSearch,
      },
      {
        id: 4,
        path: '/moose',
        title: 'Moose',
        leftIcon: BiTestTube,
      },
    ];
  }, []);

  const isSettings = React.useMemo<boolean>(() => {
    return pathname === '/settings';
  }, [pathname]);

  return (
    <Container>
      <Header>
        <StyledLogo />
      </Header>
      <NavContainer>
        <TabContainer>
          {tabs.map((tab) => (
            <LinkWrapper key={tab.id} $isActive={pathname === tab.path}>
              <Button
                variant="transparent"
                type="button"
                leftIcon={tab.leftIcon}
                fullWidth
                onClick={() => navigateTo(tab.path)}
              >
                {tab.title}
              </Button>
            </LinkWrapper>
          ))}
        </TabContainer>
        <LinkWrapper $isActive={isSettings}>
          <Button
            variant="transparent"
            type="button"
            leftIcon={HiOutlineCog}
            fullWidth
            onClick={() => navigateTo('/settings')}
          >
            Settings
          </Button>
        </LinkWrapper>
      </NavContainer>
    </Container>
  );
};

export default SideMenu;
