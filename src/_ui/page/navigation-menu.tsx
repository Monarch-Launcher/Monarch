import { House, Library, Search, Settings } from 'lucide-react';
import { Link, useLocation } from 'react-router-dom';
import styled from 'styled-components';

const Nav = styled.nav`
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  background-color: rgba(34, 34, 34, 0.8);
  backdrop-filter: blur(10px);
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  padding: 8px 16px;
  z-index: 1000;
`;

const Container = styled.div`
  display: flex;
  justify-content: space-around;
  align-items: center;
  width: 100%;
`;

const NavItem = styled(Link)<{ $active?: boolean }>`
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 15px 20px;
  min-width: 80px;
  border-radius: 8px;
  text-decoration: none;
  color: #a0a0a0;
  transition: background-color 0.3s ease, color 0.3s ease;
  text-align: center;
  background: none;
  ${({ $active, theme }) =>
    $active && `color: ${theme.colors.primary};`}
  &:hover {
    background-color: rgba(150, 150, 150, 0.2);
  }
`;

const IconStyle = styled.span`
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
`;

const Label = styled.span`
  font-size: 14px;
  margin-top: 4px;
  color: inherit;
`;

export const NavigationMenu = () => {
  const location = useLocation();

  const menuItems = [
    { icon: House, label: 'Home', path: '/' },
    { icon: Library, label: 'Library', path: '/library' },
    { icon: Search, label: 'Search', path: '/search' },
    { icon: Settings, label: 'Settings', path: '/settings' },
  ];

  return (
    <Nav>
      <Container>
        {menuItems.map((item) => {
          const isActive = location.pathname === item.path;
          return (
            <NavItem
              key={item.label}
              to={item.path}
              $active={isActive}
            >
              <IconStyle>
                <item.icon style={{ width: 28, height: 28 }} />
              </IconStyle>
              <Label>{item.label}</Label>
            </NavItem>
          );
        })}
      </Container>
    </Nav>
  );
};

export default NavigationMenu;
