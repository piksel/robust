#!/bin/bash

set -e

NAME="empty"
CC65_HOME="/b/lib/cc65"
PATH="$CC65_HOME/bin:$PATH"
# MAPPER_CONFIG="nrom_32k_vert.cfg"
MAPPER_CONFIG="nrom_32k_vert.cfg"

cc65 -Oirs $NAME.c --add-source
ca65 crt0.s
ca65 $NAME.s -g

ld65 -C $MAPPER_CONFIG -o "$NAME.nes" crt0.o "$NAME.o" nes.lib -Ln labels.txt --dbgfile dbg.txt

echo "Done!"
# rm *.o

# move /Y labels.txt BUILD\ 
# move /Y %name%.s BUILD\ 
# move /Y %name%.nes BUILD\ 

# pause

# BUILD\%name%.nes
