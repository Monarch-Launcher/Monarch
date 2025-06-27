import { House, Library, Search, Settings } from 'lucide-react';
import { Link, useLocation } from 'react-router-dom';

export const NavigationMenu = () => {
  const location = useLocation();

  const menuItems = [
    { icon: House, label: 'Home', path: '/' },
    { icon: Library, label: 'Library', path: '/library' },
    { icon: Search, label: 'Search', path: '/search' },
    { icon: Settings, label: 'Settings', path: '/settings' },
  ];

  // Inline style objects
  const navStyle: React.CSSProperties = {
    position: 'fixed',
    bottom: '0',
    left: '0',
    right: '0',
    backgroundColor: 'rgba(34, 34, 34, 0.8)', // bg-secondary with opacity
    backdropFilter: 'blur(10px)', // backdrop blur effect
    borderTop: '1px solid rgba(255, 255, 255, 0.1)', // border-white/10
    padding: '8px 16px',
  };

  const containerStyle: React.CSSProperties = {
    display: 'flex',
    justifyContent: 'space-around',
    alignItems: 'center',
    width: '100%', // ensure the items are evenly spread
  };

  const itemStyle: React.CSSProperties = {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    padding: '15px 20px', // increased padding for a wider/fatter button
    minWidth: '80px', // increase the minimum width
    borderRadius: '8px',
    textDecoration: 'none',
    color: '#a0a0a0', // text-muted-foreground
    transition: 'background-color 0.3s ease',
    textAlign: 'center', // center the text
  };

  const itemHoverStyle: React.CSSProperties = {
    backgroundColor: 'rgba(150, 150, 150, 0.2)', // hover:bg-muted
  };

  const activeItemStyle: React.CSSProperties = {
    color: 'orange', // text-primary
  };

  const iconStyle: React.CSSProperties = {
    width: '24px',
    height: '24px',
  };

  const labelStyle: React.CSSProperties = {
    fontSize: '12px',
    marginTop: '4px',
    color: 'inherit',
  };

  return (
    <nav style={navStyle}>
      <div style={containerStyle}>
        {menuItems.map((item) => {
          const isActive = location.pathname === item.path;
          return (
            <Link
              key={item.label}
              to={item.path}
              style={{
                ...itemStyle,
                ...(isActive ? activeItemStyle : {}),
              }}
              // eslint-disable-next-line no-return-assign
              onMouseEnter={(e) =>
                (e.currentTarget.style.backgroundColor =
                  itemHoverStyle.backgroundColor)
              }
              // eslint-disable-next-line no-return-assign
              onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = '')}
            >
              <item.icon style={iconStyle} />
              <span style={labelStyle}>{item.label}</span>
            </Link>
          );
        })}
      </div>
    </nav>
  );
};

export default NavigationMenu;
