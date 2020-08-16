# Dalia

## Grammar
```
file : line+ ;
line : ('[' alias ']')? path ;
alias : [a-zA-Z0-9]+ ;
path: [^\0]+ ;
```
