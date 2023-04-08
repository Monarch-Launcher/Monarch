import * as React from 'react';
import styled from 'styled-components';
import { useLocation, useNavigate } from 'react-router-dom';
import Button from '../button';

const Container = styled.div`
  background-color: ${({ theme }) => theme.colors.secondary};
  width: 12rem;
  display: flex;
  flex-direction: column;
  gap: 2rem;
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

const SideMenu = () => {
  const navigate = useNavigate();
  const { pathname } = useLocation();

  const navigateTo = React.useCallback(
    (path: string) => {
      navigate(path);
    },
    [navigate],
  );

  const isHome = React.useMemo(() => {
    return pathname === '/';
  }, [pathname]);

  const isSettings = React.useMemo(() => {
    return pathname === '/settings';
  }, [pathname]);

  const isLibrary = React.useMemo(() => {
    return pathname === '/library';
  }, [pathname]);

  const isSearch = React.useMemo(() => {
    return pathname === '/search';
  }, [pathname]);

  return (
    <Container>
      <Header>Monarch</Header>
      <NavContainer>
        <TabContainer>
          <LinkWrapper $isActive={isHome}>
            <Button
              variant="transparent"
              type="button"
              onClick={() => navigateTo('/')}
            >
              Home
            </Button>
          </LinkWrapper>
          <LinkWrapper $isActive={isLibrary}>
            <Button
              variant="transparent"
              type="button"
              onClick={() => navigateTo('/library')}
            >
              Library
            </Button>
          </LinkWrapper>
          <LinkWrapper $isActive={isSearch}>
            <Button
              variant="transparent"
              type="button"
              onClick={() => navigateTo('/search')}
            >
              Search
            </Button>
          </LinkWrapper>
        </TabContainer>
        <LinkWrapper $isActive={isSettings}>
          <Button
            variant="transparent"
            type="button"
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
