(function(a){var b={defaults:{roleAttr:"data-role",rolesSelector:"[data-role]",targetRole:".section"},init:function(){this.updateRoles();
this.bindEvents()
},updateRoles:function(d){var c={};
if(typeof d==="undefined"||d.length===0){c=a(this.defaults.rolesSelector)
}else{c=d.find(this.defaults.rolesSelector)
}var e=this;
c.each(function(){var g=a(this);
var f=g.closest(e.defaults.targetRole);
f.addClass(g.attr(e.defaults.roleAttr))
})
},bindEvents:function(){var c=this;
a(window).on("update-roles",function(e,d){c.updateRoles(d)
})
}};
b.init()
})(jQuery);