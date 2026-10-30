"""Observe competing source Tick.Post movement callbacks using real wall collision."""
def run_post_order(ctl,check,state,x,y,z):
    ctl('release')
    for module in ctl('state')['meteor']['config']['modules']['modules']:
        if module['active']: ctl('command','meteor toggle '+module['id'])
    ctl('command','gamemode survival')
    ctl('command',f'fill {x-3} {y-1} {z-3} {x+3} {y-1} {z+5} minecraft:stone')
    ctl('command',f'fill {x-3} {y} {z-3} {x+3} {y+12} {z+5} minecraft:air')
    ctl('command',f'fill {x-3} {y} {z+2} {x+3} {y+10} {z+2} minecraft:stone')
    ctl('command','meteor set flight mode Velocity')
    ctl('command','meteor set flight anti-kick-mode None')
    ctl('command','meteor set spider climb-speed 0.4')
    ctl('command','meteor toggle flight')
    ctl('command','meteor toggle spider')
    def collide(label):
        ctl('release'); ctl('tp',x+.5,y,z+.5); ctl('look',0,0); ctl('wait',3)
        ctl('key','w','down'); ctl('wait',12)
        result=state(label)['player']; ctl('release'); return result
    climb=collide('post_order_spider_last')
    check('Later Spider Post overrides Flight zero velocity at real wall',climb['velocity'][1]>.35 and climb['position'][1]>y+.3,climb)
    ctl('command','meteor toggle flight'); ctl('command','meteor toggle flight')
    hover=collide('post_order_flight_reactivated_last')
    check('Reactivated Flight Post now overrides Spider at real wall',abs(hover['velocity'][1])<1e-6 and hover['position'][1]<y+.2,hover)
    ctl('command','meteor toggle flight'); ctl('command','meteor toggle spider'); ctl('release')
