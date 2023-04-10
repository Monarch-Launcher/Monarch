import * as React from 'react';
import styled from 'styled-components';
import { useLocation, useNavigate } from 'react-router-dom';
import { BiHomeAlt, BiTestTube } from 'react-icons/bi';
import { HiOutlineSquares2X2 } from 'react-icons/hi2';
import { HiOutlineCog } from 'react-icons/hi';
import { AiOutlineSearch } from 'react-icons/ai';
import { type IconType } from 'react-icons';
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

  const tabs = React.useMemo((): Tab[] => {
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

  const isSettings = React.useMemo(() => {
    return pathname === '/settings';
  }, [pathname]);

  return (
    <Container>
      <Header>Monarch</Header>
      <NavContainer>
        <TabContainer>
          {tabs.map((tab) => (
            <LinkWrapper key={tab.id} $isActive={pathname === tab.path}>
              <Button
                variant="transparent"
                type="button"
                leftIcon={tab.leftIcon}
                width="100%"
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
            width="100%"
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
