/* A Bison parser, made by GNU Bison 2.7.  */

/* Bison implementation for Yacc-like parsers in C
   
      Copyright (C) 1984, 1989-1990, 2000-2012 Free Software Foundation, Inc.
   
   This program is free software: you can redistribute it and/or modify
   it under the terms of the GNU General Public License as published by
   the Free Software Foundation, either version 3 of the License, or
   (at your option) any later version.
   
   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU General Public License for more details.
   
   You should have received a copy of the GNU General Public License
   along with this program.  If not, see <http://www.gnu.org/licenses/>.  */

/* As a special exception, you may create a larger work that contains
   part or all of the Bison parser skeleton and distribute that work
   under terms of your choice, so long as that work isn't itself a
   parser generator using the skeleton or a modified version thereof
   as a parser skeleton.  Alternatively, if you modify or redistribute
   the parser skeleton itself, you may (at your option) remove this
   special exception, which will cause the skeleton and the resulting
   Bison output files to be licensed under the GNU General Public
   License without this special exception.
   
   This special exception was added by the Free Software Foundation in
   version 2.2 of Bison.  */

/* C LALR(1) parser skeleton written by Richard Stallman, by
   simplifying the original so-called "semantic" parser.  */

/* All symbols defined below should begin with yy or YY, to avoid
   infringing on user name space.  This should be done even for local
   variables, as they might otherwise be expanded by user macros.
   There are some unavoidable exceptions within include files to
   define necessary library symbols; they are noted "INFRINGES ON
   USER NAME SPACE" below.  */

/* Identify Bison output.  */
#define YYBISON 1

/* Bison version.  */
#define YYBISON_VERSION "2.7"

/* Skeleton name.  */
#define YYSKELETON_NAME "yacc.c"

/* Pure parsers.  */
#define YYPURE 0

/* Push parsers.  */
#define YYPUSH 0

/* Pull parsers.  */
#define YYPULL 1




/* Copy the first part of user declarations.  */
/* Line 371 of yacc.c  */
#line 1 "iris.y"

#include "IrisCompilerDev.h"
#include <sstream>
using namespace std;
//#define YYDEBUG 1

extern int yylex();

int yyerror(char const *str)
{
	extern char *yytext;

	string strMessage = "<Irregular : SyntaxIrregular>\n  Irregular-happened Report : Oh! Master, an Irregular has happened and Iris is not clever and dosen't kown how to deal with it. Can you please cheak it yourself? \n";
	string strIrregularMessage = ">The Irregular name is ";
	stringstream ssStream;
	ssStream << IrisCompiler::CurrentCompiler()->GetCurrentLineNumber();
	string strLinenoMessage = ">and happened at line " + ssStream.str() + " file " + IrisCompiler::CurrentCompiler()->GetCurrentFileName() + " where token is " + string(yytext) + ".";

	strMessage += strIrregularMessage + "SyntaxIrregular," + "\n" + strLinenoMessage + "\n";

	//fprintf(stderr, "parser error near %s\n", yytext);
	//fprintf(stderr, strMessage.c_str());
	IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorDirectly(strMessage);
	return 0;
}

/* Line 371 of yacc.c  */
#line 95 "y.tab.c"

# ifndef YY_NULL
#  if defined __cplusplus && 201103L <= __cplusplus
#   define YY_NULL nullptr
#  else
#   define YY_NULL 0
#  endif
# endif

/* Enabling verbose error messages.  */
#ifdef YYERROR_VERBOSE
# undef YYERROR_VERBOSE
# define YYERROR_VERBOSE 1
#else
# define YYERROR_VERBOSE 0
#endif

/* In a future release of Bison, this section will be replaced
   by #include "y.tab.h".  */
#ifndef YY_YY_Y_TAB_H_INCLUDED
# define YY_YY_Y_TAB_H_INCLUDED
/* Enabling traces.  */
#ifndef YYDEBUG
# define YYDEBUG 0
#endif
#if YYDEBUG
extern int yydebug;
#endif

/* Tokens.  */
#ifndef YYTOKENTYPE
# define YYTOKENTYPE
   /* Put the tokens into the symbol table, so that GDB and other debuggers
      know about them.  */
   enum yytokentype {
     INTEGER = 258,
     FLOAT = 259,
     STRING = 260,
     IDENTIFIER = 261,
     FUNCTION = 262,
     CLASS = 263,
     MODULE = 264,
     INTERFACE = 265,
     EXTENDS = 266,
     INVOLVES = 267,
     JOINTS = 268,
     PERSONAL = 269,
     RELATIVE = 270,
     EVERYONE = 271,
     GET = 272,
     SET = 273,
     GSET = 274,
     IF = 275,
     ELSE = 276,
     ELSEIF = 277,
     FOR = 278,
     IN = 279,
     SWITCH = 280,
     WHEN = 281,
     CAST = 282,
     ITERATOR = 283,
     SUPER = 284,
     BREAK = 285,
     CONTINUE = 286,
     RETURN = 287,
     BLOCK = 288,
     SELF = 289,
     WITH = 290,
     WITHOUT = 291,
     ORDER = 292,
     SERVE = 293,
     IGNORE = 294,
     GROAN = 295,
     TRUE = 296,
     FALSE = 297,
     NIL = 298,
     DOT = 299,
     ALIAS = 300,
     RETRY = 301,
     REDO = 302,
     GOTO = 303,
     CONST = 304,
     GLOBAL = 305,
     STATIC = 306,
     LP = 307,
     RP = 308,
     LC = 309,
     RC = 310,
     LB = 311,
     RB = 312,
     SEMICOLON = 313,
     COMMA = 314,
     LOGICAL_AND = 315,
     LOGICAL_OR = 316,
     LOGICAL_NOT = 317,
     ASSIGN = 318,
     ADD = 319,
     SUB = 320,
     MUL = 321,
     DIV = 322,
     MOD = 323,
     POWER = 324,
     BIT_AND = 325,
     BIT_OR = 326,
     BIT_XOR = 327,
     BIT_NOT = 328,
     ASSIGN_ADD = 329,
     ASSIGN_SUB = 330,
     ASSIGN_MUL = 331,
     ASSIGN_DIV = 332,
     ASSIGN_MOD = 333,
     ASSIGN_BIT_AND = 334,
     ASSIGN_BIT_OR = 335,
     ASSIGN_BIT_XOR = 336,
     BIT_SHR = 337,
     EQ = 338,
     NE = 339,
     GT = 340,
     GE = 341,
     LT = 342,
     LE = 343,
     BIT_SHL = 344,
     BIT_SAR = 345,
     BIT_SAL = 346,
     ASSIGN_BIT_SHR = 347,
     ASSIGN_BIT_SHL = 348,
     ASSIGN_BIT_SAR = 349,
     ASSIGN_BIT_SAL = 350,
     ITER = 351,
     FILED = 352,
     SHARP = 353,
     CONLON = 354
   };
#endif
/* Tokens.  */
#define INTEGER 258
#define FLOAT 259
#define STRING 260
#define IDENTIFIER 261
#define FUNCTION 262
#define CLASS 263
#define MODULE 264
#define INTERFACE 265
#define EXTENDS 266
#define INVOLVES 267
#define JOINTS 268
#define PERSONAL 269
#define RELATIVE 270
#define EVERYONE 271
#define GET 272
#define SET 273
#define GSET 274
#define IF 275
#define ELSE 276
#define ELSEIF 277
#define FOR 278
#define IN 279
#define SWITCH 280
#define WHEN 281
#define CAST 282
#define ITERATOR 283
#define SUPER 284
#define BREAK 285
#define CONTINUE 286
#define RETURN 287
#define BLOCK 288
#define SELF 289
#define WITH 290
#define WITHOUT 291
#define ORDER 292
#define SERVE 293
#define IGNORE 294
#define GROAN 295
#define TRUE 296
#define FALSE 297
#define NIL 298
#define DOT 299
#define ALIAS 300
#define RETRY 301
#define REDO 302
#define GOTO 303
#define CONST 304
#define GLOBAL 305
#define STATIC 306
#define LP 307
#define RP 308
#define LC 309
#define RC 310
#define LB 311
#define RB 312
#define SEMICOLON 313
#define COMMA 314
#define LOGICAL_AND 315
#define LOGICAL_OR 316
#define LOGICAL_NOT 317
#define ASSIGN 318
#define ADD 319
#define SUB 320
#define MUL 321
#define DIV 322
#define MOD 323
#define POWER 324
#define BIT_AND 325
#define BIT_OR 326
#define BIT_XOR 327
#define BIT_NOT 328
#define ASSIGN_ADD 329
#define ASSIGN_SUB 330
#define ASSIGN_MUL 331
#define ASSIGN_DIV 332
#define ASSIGN_MOD 333
#define ASSIGN_BIT_AND 334
#define ASSIGN_BIT_OR 335
#define ASSIGN_BIT_XOR 336
#define BIT_SHR 337
#define EQ 338
#define NE 339
#define GT 340
#define GE 341
#define LT 342
#define LE 343
#define BIT_SHL 344
#define BIT_SAR 345
#define BIT_SAL 346
#define ASSIGN_BIT_SHR 347
#define ASSIGN_BIT_SHL 348
#define ASSIGN_BIT_SAR 349
#define ASSIGN_BIT_SAL 350
#define ITER 351
#define FILED 352
#define SHARP 353
#define CONLON 354



#if ! defined YYSTYPE && ! defined YYSTYPE_IS_DECLARED
typedef union YYSTYPE
{
/* Line 387 of yacc.c  */
#line 28 "iris.y"

	IrisIdentifier* m_pIdentifier = nullptr;
	IrisList<IrisIdentifier*>* m_pIdentifierList;
	IrisExpression* m_pExpression; 
	IrisList<IrisExpression*>* m_pExpressionList;
	IrisStatement* m_pStatement;
	IrisList<IrisStatement*>* m_pStatementList;
	IrisBinaryExpressionType m_eExpressionType;
	IrisBlock* m_pBlock;
	IrisFunctionHeader* m_pFunctionHeader;
	IrisConditionIfStatement* m_pConditionIf;
	IrisLoopIfStatement* m_pLoopIf;
	IrisElseIf* m_pElseIf;
	IrisList<IrisElseIf*>* m_pElseIfList;
	IrisWhen* m_pWhen;
	IrisList<IrisWhen*>* m_pWhenList;
	IrisSwitchBlock* m_pSwitchBlock;
	IrisHashPair* m_pHashPair;
	IrisList<IrisHashPair*>* m_pHashPairList;
	IrisClosureBlockLiteral* m_pClosureBlockLiteral;


/* Line 387 of yacc.c  */
#line 359 "y.tab.c"
} YYSTYPE;
# define YYSTYPE_IS_TRIVIAL 1
# define yystype YYSTYPE /* obsolescent; will be withdrawn */
# define YYSTYPE_IS_DECLARED 1
#endif

extern YYSTYPE yylval;

#ifdef YYPARSE_PARAM
#if defined __STDC__ || defined __cplusplus
int yyparse (void *YYPARSE_PARAM);
#else
int yyparse ();
#endif
#else /* ! YYPARSE_PARAM */
#if defined __STDC__ || defined __cplusplus
int yyparse (void);
#else
int yyparse ();
#endif
#endif /* ! YYPARSE_PARAM */

#endif /* !YY_YY_Y_TAB_H_INCLUDED  */

/* Copy the second part of user declarations.  */

/* Line 390 of yacc.c  */
#line 387 "y.tab.c"

#ifdef short
# undef short
#endif

#ifdef YYTYPE_UINT8
typedef YYTYPE_UINT8 yytype_uint8;
#else
typedef unsigned char yytype_uint8;
#endif

#ifdef YYTYPE_INT8
typedef YYTYPE_INT8 yytype_int8;
#elif (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
typedef signed char yytype_int8;
#else
typedef short int yytype_int8;
#endif

#ifdef YYTYPE_UINT16
typedef YYTYPE_UINT16 yytype_uint16;
#else
typedef unsigned short int yytype_uint16;
#endif

#ifdef YYTYPE_INT16
typedef YYTYPE_INT16 yytype_int16;
#else
typedef short int yytype_int16;
#endif

#ifndef YYSIZE_T
# ifdef __SIZE_TYPE__
#  define YYSIZE_T __SIZE_TYPE__
# elif defined size_t
#  define YYSIZE_T size_t
# elif ! defined YYSIZE_T && (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
#  include <stddef.h> /* INFRINGES ON USER NAME SPACE */
#  define YYSIZE_T size_t
# else
#  define YYSIZE_T unsigned int
# endif
#endif

#define YYSIZE_MAXIMUM ((YYSIZE_T) -1)

#ifndef YY_
# if defined YYENABLE_NLS && YYENABLE_NLS
#  if ENABLE_NLS
#   include <libintl.h> /* INFRINGES ON USER NAME SPACE */
#   define YY_(Msgid) dgettext ("bison-runtime", Msgid)
#  endif
# endif
# ifndef YY_
#  define YY_(Msgid) Msgid
# endif
#endif

/* Suppress unused-variable warnings by "using" E.  */
#if ! defined lint || defined __GNUC__
# define YYUSE(E) ((void) (E))
#else
# define YYUSE(E) /* empty */
#endif

/* Identity function, used to suppress warnings about constant conditions.  */
#ifndef lint
# define YYID(N) (N)
#else
#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
static int
YYID (int yyi)
#else
static int
YYID (yyi)
    int yyi;
#endif
{
  return yyi;
}
#endif

#if ! defined yyoverflow || YYERROR_VERBOSE

/* The parser invokes alloca or malloc; define the necessary symbols.  */

# ifdef YYSTACK_USE_ALLOCA
#  if YYSTACK_USE_ALLOCA
#   ifdef __GNUC__
#    define YYSTACK_ALLOC __builtin_alloca
#   elif defined __BUILTIN_VA_ARG_INCR
#    include <alloca.h> /* INFRINGES ON USER NAME SPACE */
#   elif defined _AIX
#    define YYSTACK_ALLOC __alloca
#   elif defined _MSC_VER
#    include <malloc.h> /* INFRINGES ON USER NAME SPACE */
#    define alloca _alloca
#   else
#    define YYSTACK_ALLOC alloca
#    if ! defined _ALLOCA_H && ! defined EXIT_SUCCESS && (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
#     include <stdlib.h> /* INFRINGES ON USER NAME SPACE */
      /* Use EXIT_SUCCESS as a witness for stdlib.h.  */
#     ifndef EXIT_SUCCESS
#      define EXIT_SUCCESS 0
#     endif
#    endif
#   endif
#  endif
# endif

# ifdef YYSTACK_ALLOC
   /* Pacify GCC's `empty if-body' warning.  */
#  define YYSTACK_FREE(Ptr) do { /* empty */; } while (YYID (0))
#  ifndef YYSTACK_ALLOC_MAXIMUM
    /* The OS might guarantee only one guard page at the bottom of the stack,
       and a page size can be as small as 4096 bytes.  So we cannot safely
       invoke alloca (N) if N exceeds 4096.  Use a slightly smaller number
       to allow for a few compiler-allocated temporary stack slots.  */
#   define YYSTACK_ALLOC_MAXIMUM 4032 /* reasonable circa 2006 */
#  endif
# else
#  define YYSTACK_ALLOC YYMALLOC
#  define YYSTACK_FREE YYFREE
#  ifndef YYSTACK_ALLOC_MAXIMUM
#   define YYSTACK_ALLOC_MAXIMUM YYSIZE_MAXIMUM
#  endif
#  if (defined __cplusplus && ! defined EXIT_SUCCESS \
       && ! ((defined YYMALLOC || defined malloc) \
	     && (defined YYFREE || defined free)))
#   include <stdlib.h> /* INFRINGES ON USER NAME SPACE */
#   ifndef EXIT_SUCCESS
#    define EXIT_SUCCESS 0
#   endif
#  endif
#  ifndef YYMALLOC
#   define YYMALLOC malloc
#   if ! defined malloc && ! defined EXIT_SUCCESS && (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
void *malloc (YYSIZE_T); /* INFRINGES ON USER NAME SPACE */
#   endif
#  endif
#  ifndef YYFREE
#   define YYFREE free
#   if ! defined free && ! defined EXIT_SUCCESS && (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
void free (void *); /* INFRINGES ON USER NAME SPACE */
#   endif
#  endif
# endif
#endif /* ! defined yyoverflow || YYERROR_VERBOSE */


#if (! defined yyoverflow \
     && (! defined __cplusplus \
	 || (defined YYSTYPE_IS_TRIVIAL && YYSTYPE_IS_TRIVIAL)))

/* A type that is properly aligned for any stack member.  */
union yyalloc
{
  yytype_int16 yyss_alloc;
  YYSTYPE yyvs_alloc;
};

/* The size of the maximum gap between one aligned stack and the next.  */
# define YYSTACK_GAP_MAXIMUM (sizeof (union yyalloc) - 1)

/* The size of an array large to enough to hold all stacks, each with
   N elements.  */
# define YYSTACK_BYTES(N) \
     ((N) * (sizeof (yytype_int16) + sizeof (YYSTYPE)) \
      + YYSTACK_GAP_MAXIMUM)

# define YYCOPY_NEEDED 1

/* Relocate STACK from its old location to the new one.  The
   local variables YYSIZE and YYSTACKSIZE give the old and new number of
   elements in the stack, and YYPTR gives the new location of the
   stack.  Advance YYPTR to a properly aligned location for the next
   stack.  */
# define YYSTACK_RELOCATE(Stack_alloc, Stack)				\
    do									\
      {									\
	YYSIZE_T yynewbytes;						\
	YYCOPY (&yyptr->Stack_alloc, Stack, yysize);			\
	Stack = &yyptr->Stack_alloc;					\
	yynewbytes = yystacksize * sizeof (*Stack) + YYSTACK_GAP_MAXIMUM; \
	yyptr += yynewbytes / sizeof (*yyptr);				\
      }									\
    while (YYID (0))

#endif

#if defined YYCOPY_NEEDED && YYCOPY_NEEDED
/* Copy COUNT objects from SRC to DST.  The source and destination do
   not overlap.  */
# ifndef YYCOPY
#  if defined __GNUC__ && 1 < __GNUC__
#   define YYCOPY(Dst, Src, Count) \
      __builtin_memcpy (Dst, Src, (Count) * sizeof (*(Src)))
#  else
#   define YYCOPY(Dst, Src, Count)              \
      do                                        \
        {                                       \
          YYSIZE_T yyi;                         \
          for (yyi = 0; yyi < (Count); yyi++)   \
            (Dst)[yyi] = (Src)[yyi];            \
        }                                       \
      while (YYID (0))
#  endif
# endif
#endif /* !YYCOPY_NEEDED */

/* YYFINAL -- State number of the termination state.  */
#define YYFINAL  77
/* YYLAST -- Last index in YYTABLE.  */
#define YYLAST   1300

/* YYNTOKENS -- Number of terminals.  */
#define YYNTOKENS  100
/* YYNNTS -- Number of nonterminals.  */
#define YYNNTS  93
/* YYNRULES -- Number of rules.  */
#define YYNRULES  318
/* YYNRULES -- Number of states.  */
#define YYNSTATES  577

/* YYTRANSLATE(YYLEX) -- Bison symbol number corresponding to YYLEX.  */
#define YYUNDEFTOK  2
#define YYMAXUTOK   354

#define YYTRANSLATE(YYX)						\
  ((unsigned int) (YYX) <= YYMAXUTOK ? yytranslate[YYX] : YYUNDEFTOK)

/* YYTRANSLATE[YYLEX] -- Bison symbol number corresponding to YYLEX.  */
static const yytype_uint8 yytranslate[] =
{
       0,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     2,     2,     2,     2,
       2,     2,     2,     2,     2,     2,     1,     2,     3,     4,
       5,     6,     7,     8,     9,    10,    11,    12,    13,    14,
      15,    16,    17,    18,    19,    20,    21,    22,    23,    24,
      25,    26,    27,    28,    29,    30,    31,    32,    33,    34,
      35,    36,    37,    38,    39,    40,    41,    42,    43,    44,
      45,    46,    47,    48,    49,    50,    51,    52,    53,    54,
      55,    56,    57,    58,    59,    60,    61,    62,    63,    64,
      65,    66,    67,    68,    69,    70,    71,    72,    73,    74,
      75,    76,    77,    78,    79,    80,    81,    82,    83,    84,
      85,    86,    87,    88,    89,    90,    91,    92,    93,    94,
      95,    96,    97,    98,    99
};

#if YYDEBUG
/* YYPRHS[YYN] -- Index of the first RHS symbol of rule number YYN in
   YYRHS.  */
static const yytype_uint16 yyprhs[] =
{
       0,     0,     3,     5,     8,    11,    13,    15,    17,    19,
      21,    23,    25,    27,    29,    31,    33,    35,    42,    45,
      49,    51,    54,    57,    59,    61,    63,    65,    67,    69,
      71,    73,    75,    77,    79,    81,    83,    85,    86,    89,
      92,    93,    96,    97,   100,   102,   104,   108,   112,   118,
     126,   132,   141,   147,   153,   161,   167,   175,   181,   189,
     194,   197,   201,   203,   206,   209,   211,   213,   215,   217,
     219,   221,   226,   229,   233,   235,   238,   241,   243,   245,
     247,   249,   251,   253,   255,   257,   259,   261,   263,   265,
     271,   279,   285,   293,   299,   307,   310,   317,   320,   324,
     326,   329,   332,   334,   336,   338,   340,   342,   344,   346,
     348,   350,   353,   357,   359,   362,   365,   367,   369,   371,
     373,   375,   377,   379,   381,   386,   392,   401,   408,   415,
     423,   434,   443,   446,   451,   457,   460,   464,   466,   468,
     470,   472,   474,   476,   478,   480,   482,   484,   486,   488,
     490,   492,   494,   496,   498,   500,   502,   504,   506,   508,
     510,   512,   514,   520,   528,   535,   544,   546,   549,   555,
     564,   565,   568,   574,   578,   584,   586,   589,   595,   603,
     615,   624,   627,   631,   633,   636,   639,   641,   643,   645,
     647,   649,   651,   653,   654,   659,   662,   666,   669,   672,
     678,   685,   695,   703,   709,   712,   717,   723,   728,   734,
     737,   741,   743,   746,   748,   752,   754,   756,   758,   760,
     762,   764,   766,   768,   770,   772,   774,   776,   778,   780,
     784,   786,   790,   792,   796,   798,   802,   804,   808,   810,
     814,   818,   820,   824,   828,   832,   836,   838,   842,   846,
     850,   854,   856,   860,   864,   866,   870,   874,   878,   880,
     884,   886,   889,   892,   895,   898,   900,   905,   912,   920,
     925,   931,   935,   936,   939,   943,   953,   955,   958,   961,
     963,   965,   967,   969,   971,   973,   977,   979,   981,   983,
     985,   987,   989,   991,   993,   995,   997,   999,  1002,  1006,
    1009,  1012,  1016,  1021,  1024,  1028,  1030,  1034,  1036,  1040,
    1043,  1047,  1054,  1056,  1059,  1064,  1070,  1076,  1082
};

/* YYRHS -- A `-1'-separated list of the rules' RHS.  */
static const yytype_int16 yyrhs[] =
{
     101,     0,    -1,   102,    -1,   101,   102,    -1,    58,   165,
      -1,   128,    -1,   155,    -1,   139,    -1,   156,    -1,   157,
      -1,   145,    -1,   121,    -1,   103,    -1,   117,    -1,   149,
      -1,   150,    -1,   159,    -1,     8,     6,   107,   109,   108,
     104,    -1,    54,    55,    -1,    54,   105,    55,    -1,   106,
      -1,   105,   106,    -1,    58,   165,    -1,   145,    -1,   121,
      -1,   128,    -1,   136,    -1,   139,    -1,   111,    -1,   112,
      -1,   113,    -1,   114,    -1,   115,    -1,   116,    -1,   149,
      -1,   150,    -1,   159,    -1,    -1,    11,   184,    -1,    11,
       6,    -1,    -1,    13,   110,    -1,    -1,    12,   110,    -1,
       6,    -1,   184,    -1,   110,    59,     6,    -1,   110,    59,
     184,    -1,    58,    17,    56,     6,    57,    -1,    17,    56,
       6,    57,    52,    53,   163,    -1,    58,    18,    56,     6,
      57,    -1,    18,    56,     6,    57,    52,     6,    53,   163,
      -1,    58,    19,    56,     6,    57,    -1,    58,    16,    56,
       6,    57,    -1,    58,    16,    56,    34,    44,     6,    57,
      -1,    58,    15,    56,     6,    57,    -1,    58,    15,    56,
      34,    44,     6,    57,    -1,    58,    14,    56,     6,    57,
      -1,    58,    14,    56,    34,    44,     6,    57,    -1,    10,
       6,   108,   118,    -1,    54,    55,    -1,    54,   119,    55,
      -1,   120,    -1,   119,   120,    -1,    58,   165,    -1,   145,
      -1,   139,    -1,   158,    -1,   149,    -1,   150,    -1,   159,
      -1,     9,     6,   109,   122,    -1,    54,    55,    -1,    54,
     123,    55,    -1,   124,    -1,   123,   124,    -1,    58,   165,
      -1,   145,    -1,   121,    -1,   128,    -1,   139,    -1,   103,
      -1,   125,    -1,   126,    -1,   127,    -1,   117,    -1,   149,
      -1,   150,    -1,   159,    -1,    58,    16,    56,     6,    57,
      -1,    58,    16,    56,    34,    44,     6,    57,    -1,    58,
      15,    56,     6,    57,    -1,    58,    15,    56,    34,    44,
       6,    57,    -1,    58,    14,    56,     6,    57,    -1,    58,
      14,    56,    34,    44,     6,    57,    -1,   135,   129,    -1,
     135,   129,    35,   132,    36,   132,    -1,    54,    55,    -1,
      54,   130,    55,    -1,   131,    -1,   130,   131,    -1,    58,
     165,    -1,   145,    -1,   139,    -1,   103,    -1,   149,    -1,
     150,    -1,   159,    -1,   155,    -1,   160,    -1,   162,    -1,
      54,    55,    -1,    54,   133,    55,    -1,   134,    -1,   133,
     134,    -1,    58,   165,    -1,   145,    -1,   139,    -1,   103,
      -1,   149,    -1,   150,    -1,   159,    -1,   155,    -1,   161,
      -1,     7,     6,    52,    53,    -1,     7,     6,    52,   187,
      53,    -1,     7,     6,    52,   187,    59,    66,     6,    53,
      -1,     7,     6,    52,    66,     6,    53,    -1,     7,    34,
      44,     6,    52,    53,    -1,     7,    34,    44,     6,    52,
     187,    53,    -1,     7,    34,    44,     6,    52,   187,    59,
      66,     6,    53,    -1,     7,    34,    44,     6,    52,    66,
       6,    53,    -1,   137,   129,    -1,     7,   138,    52,    53,
      -1,     7,   138,    52,   187,    53,    -1,    56,    57,    -1,
      56,    57,    63,    -1,    60,    -1,    61,    -1,    62,    -1,
      83,    -1,    84,    -1,    85,    -1,    86,    -1,    87,    -1,
      88,    -1,    64,    -1,    65,    -1,    66,    -1,    67,    -1,
      68,    -1,    69,    -1,    70,    -1,    71,    -1,    72,    -1,
      73,    -1,    82,    -1,    89,    -1,    90,    -1,    91,    -1,
     140,    -1,   143,    -1,    20,    52,   165,    53,   163,    -1,
      20,    52,   165,    53,   163,    21,   163,    -1,    20,    52,
     165,    53,   163,   141,    -1,    20,    52,   165,    53,   163,
     141,    21,   163,    -1,   142,    -1,   141,   142,    -1,    22,
      52,   165,    53,   163,    -1,    20,    52,   165,    59,   165,
     144,    53,   163,    -1,    -1,    59,     6,    -1,    25,    52,
     165,    53,   146,    -1,    54,   147,    55,    -1,    54,   147,
      21,   163,    55,    -1,   148,    -1,   147,   148,    -1,    26,
      52,   188,    53,   163,    -1,    23,    52,     6,    24,   165,
      53,   163,    -1,    23,    52,    52,     6,    59,     6,    53,
      24,   165,    53,   163,    -1,    37,   163,    38,    52,     6,
      53,   151,   154,    -1,    54,    55,    -1,    54,   152,    55,
      -1,   153,    -1,   152,   153,    -1,    58,   165,    -1,   145,
      -1,   121,    -1,   128,    -1,   139,    -1,   149,    -1,   150,
      -1,   159,    -1,    -1,    39,    54,   152,    55,    -1,    58,
      32,    -1,    58,    32,   165,    -1,    58,    30,    -1,    58,
      31,    -1,    58,     7,     6,    52,    53,    -1,    58,     7,
       6,    52,   187,    53,    -1,    58,     7,     6,    52,   187,
      59,    66,     6,    53,    -1,    58,     7,     6,    52,    66,
       6,    53,    -1,    58,    40,    52,   165,    53,    -1,    58,
      33,    -1,    58,    27,    52,    53,    -1,    58,    27,    52,
     188,    53,    -1,    58,    29,    52,    53,    -1,    58,    29,
      52,   188,    53,    -1,    54,    55,    -1,    54,   164,    55,
      -1,   102,    -1,   164,   102,    -1,   167,    -1,   179,   166,
     165,    -1,    63,    -1,    74,    -1,    75,    -1,    76,    -1,
      77,    -1,    78,    -1,    79,    -1,    80,    -1,    81,    -1,
      92,    -1,    93,    -1,    94,    -1,    95,    -1,   168,    -1,
     167,    61,   168,    -1,   169,    -1,   168,    60,   169,    -1,
     170,    -1,   169,    71,   170,    -1,   171,    -1,   170,    72,
     171,    -1,   172,    -1,   171,    70,   172,    -1,   173,    -1,
     172,    83,   173,    -1,   172,    84,   173,    -1,   174,    -1,
     173,    85,   174,    -1,   173,    86,   174,    -1,   173,    87,
     174,    -1,   173,    88,   174,    -1,   175,    -1,   174,    82,
     175,    -1,   174,    89,   175,    -1,   174,    90,   175,    -1,
     174,    91,   175,    -1,   176,    -1,   175,    64,   176,    -1,
     175,    65,   176,    -1,   177,    -1,   176,    66,   177,    -1,
     176,    67,   177,    -1,   176,    68,   177,    -1,   178,    -1,
     177,    69,   178,    -1,   179,    -1,    62,   178,    -1,    73,
     178,    -1,    65,   178,    -1,    64,   178,    -1,   183,    -1,
     179,    56,   165,    57,    -1,   179,    44,     6,    52,    53,
     180,    -1,   179,    44,     6,    52,   188,    53,   180,    -1,
       6,    52,    53,   180,    -1,     6,    52,   188,    53,   180,
      -1,   179,    44,     6,    -1,    -1,    54,    55,    -1,    54,
     181,    55,    -1,    54,    28,    96,    56,   187,    57,    99,
     181,    55,    -1,   182,    -1,   181,   182,    -1,    58,   165,
      -1,   145,    -1,   139,    -1,   149,    -1,   150,    -1,   159,
      -1,   184,    -1,    52,   165,    53,    -1,     6,    -1,     3,
      -1,     4,    -1,     5,    -1,    41,    -1,    42,    -1,    43,
      -1,    34,    -1,   185,    -1,   189,    -1,   192,    -1,   186,
       6,    -1,    97,   186,     6,    -1,    97,     6,    -1,    56,
      57,    -1,    56,   188,    57,    -1,    56,   188,    59,    57,
      -1,     6,    97,    -1,   186,     6,    97,    -1,     6,    -1,
     187,    59,     6,    -1,   165,    -1,   188,    59,   165,    -1,
      54,    55,    -1,    54,   190,    55,    -1,    54,   190,   165,
      96,   165,    55,    -1,   191,    -1,   190,   191,    -1,   165,
      96,   165,    59,    -1,    52,   165,    96,   165,    53,    -1,
      52,   165,    96,   165,    57,    -1,    56,   165,    96,   165,
      57,    -1,    56,   165,    96,   165,    53,    -1
};

/* YYRLINE[YYN] -- source line where rule number YYN was defined.  */
static const yytype_uint16 yyrline[] =
{
       0,   109,   109,   112,   118,   123,   124,   125,   126,   127,
     128,   129,   130,   131,   132,   133,   134,   139,   147,   150,
     156,   161,   168,   173,   174,   175,   176,   177,   178,   179,
     180,   181,   182,   183,   184,   185,   186,   190,   193,   196,
     202,   205,   211,   214,   220,   225,   230,   234,   242,   247,
     256,   261,   270,   279,   284,   292,   297,   305,   310,   318,
     326,   329,   335,   340,   347,   352,   353,   354,   355,   356,
     357,   362,   370,   373,   379,   384,   391,   396,   397,   398,
     399,   400,   401,   402,   403,   404,   405,   406,   407,   412,
     417,   425,   430,   438,   443,   451,   456,   464,   467,   473,
     478,   485,   490,   491,   492,   493,   494,   495,   496,   497,
     498,   502,   505,   511,   516,   523,   528,   529,   530,   531,
     532,   533,   534,   535,   539,   542,   545,   548,   551,   554,
     557,   560,   566,   574,   577,   588,   591,   594,   597,   600,
     603,   606,   609,   612,   615,   618,   621,   624,   627,   630,
     633,   636,   639,   642,   645,   648,   651,   654,   657,   660,
     667,   671,   678,   682,   687,   691,   698,   703,   710,   717,
     724,   727,   734,   742,   746,   753,   758,   765,   773,   778,
     787,   795,   798,   804,   809,   816,   821,   822,   823,   824,
     825,   826,   827,   832,   835,   843,   848,   857,   865,   874,
     879,   884,   889,   898,   907,   916,   921,   930,   935,   943,
     947,   953,   958,   966,   967,   974,   977,   980,   983,   986,
     989,   992,   995,   998,  1001,  1004,  1007,  1010,  1016,  1017,
    1024,  1025,  1032,  1033,  1040,  1041,  1048,  1049,  1056,  1057,
    1061,  1068,  1069,  1073,  1077,  1081,  1088,  1089,  1093,  1097,
    1101,  1108,  1109,  1113,  1120,  1121,  1125,  1129,  1136,  1137,
    1144,  1145,  1149,  1153,  1157,  1164,  1165,  1169,  1173,  1187,
    1191,  1195,  1208,  1211,  1214,  1217,  1223,  1228,  1235,  1240,
    1241,  1242,  1243,  1244,  1248,  1249,  1252,  1256,  1260,  1264,
    1268,  1272,  1276,  1280,  1284,  1285,  1286,  1290,  1294,  1298,
    1305,  1309,  1313,  1321,  1326,  1334,  1339,  1346,  1351,  1358,
    1362,  1366,  1373,  1378,  1384,  1389,  1393,  1397,  1401
};
#endif

#if YYDEBUG || YYERROR_VERBOSE || 0
/* YYTNAME[SYMBOL-NUM] -- String name of the symbol SYMBOL-NUM.
   First, the terminals, then, starting at YYNTOKENS, nonterminals.  */
static const char *const yytname[] =
{
  "$end", "error", "$undefined", "INTEGER", "FLOAT", "STRING",
  "IDENTIFIER", "FUNCTION", "CLASS", "MODULE", "INTERFACE", "EXTENDS",
  "INVOLVES", "JOINTS", "PERSONAL", "RELATIVE", "EVERYONE", "GET", "SET",
  "GSET", "IF", "ELSE", "ELSEIF", "FOR", "IN", "SWITCH", "WHEN", "CAST",
  "ITERATOR", "SUPER", "BREAK", "CONTINUE", "RETURN", "BLOCK", "SELF",
  "WITH", "WITHOUT", "ORDER", "SERVE", "IGNORE", "GROAN", "TRUE", "FALSE",
  "NIL", "DOT", "ALIAS", "RETRY", "REDO", "GOTO", "CONST", "GLOBAL",
  "STATIC", "LP", "RP", "LC", "RC", "LB", "RB", "SEMICOLON", "COMMA",
  "LOGICAL_AND", "LOGICAL_OR", "LOGICAL_NOT", "ASSIGN", "ADD", "SUB",
  "MUL", "DIV", "MOD", "POWER", "BIT_AND", "BIT_OR", "BIT_XOR", "BIT_NOT",
  "ASSIGN_ADD", "ASSIGN_SUB", "ASSIGN_MUL", "ASSIGN_DIV", "ASSIGN_MOD",
  "ASSIGN_BIT_AND", "ASSIGN_BIT_OR", "ASSIGN_BIT_XOR", "BIT_SHR", "EQ",
  "NE", "GT", "GE", "LT", "LE", "BIT_SHL", "BIT_SAR", "BIT_SAL",
  "ASSIGN_BIT_SHR", "ASSIGN_BIT_SHL", "ASSIGN_BIT_SAR", "ASSIGN_BIT_SAL",
  "ITER", "FILED", "SHARP", "CONLON", "$accept", "translation_unit",
  "statement", "class_statement", "class_block",
  "class_involve_statement_list", "class_involve_statement",
  "extends_field", "interface_field", "module_field",
  "field_identifier_list", "getter_statement", "setter_statement",
  "getter_setter_statement", "class_everyone_statement",
  "class_relative_statement", "class_personal_statement",
  "interface_statement", "interface_block",
  "interface_involve_statement_list", "interface_involve_statement",
  "module_statement", "module_block", "module_involve_statement_list",
  "module_involve_statement", "module_everyone_statement",
  "module_relative_statement", "module_personal_statement",
  "function_statement", "function_block",
  "function_involve_statement_list", "function_involve_statement",
  "with_without_block", "with_without_involve_statement_list",
  "with_without_involve_statement", "function_header",
  "function_operator_statement", "function_operator_header", "operator",
  "if_statement", "condition_if", "elseif_list", "elseif", "loop_if",
  "loop_time", "switch_statement", "switch_block", "when_list", "when",
  "for_statement", "order_statement", "order_block",
  "order_involve_statement_list", "order_involve_statement",
  "ignore_block", "return_statement", "break_statement",
  "continue_statement", "interface_func_statement", "groan_statement",
  "block_statement", "cast_statement", "super_statement", "block",
  "statement_list", "expression", "assign_symbol", "logic_or_expression",
  "logic_and_expression", "logic_bit_or_expression",
  "logic_bit_xor_expression", "logic_bit_and_expression",
  "logic_equal_compare_expression", "logic_not_equal_expression",
  "logic_shift_expression", "add_sub_expression", "mul_div_mod_expression",
  "power_expression", "unary_expression", "top_expression",
  "closure_block", "closure_involve_statement_list",
  "closure_involve_statement", "primary_expression",
  "identifier_with_field", "array_literal", "field_identifier",
  "identifier_list", "expression_list", "hash_literal", "hash_pair_list",
  "hash_pair", "range_literal", YY_NULL
};
#endif

# ifdef YYPRINT
/* YYTOKNUM[YYLEX-NUM] -- Internal token number corresponding to
   token YYLEX-NUM.  */
static const yytype_uint16 yytoknum[] =
{
       0,   256,   257,   258,   259,   260,   261,   262,   263,   264,
     265,   266,   267,   268,   269,   270,   271,   272,   273,   274,
     275,   276,   277,   278,   279,   280,   281,   282,   283,   284,
     285,   286,   287,   288,   289,   290,   291,   292,   293,   294,
     295,   296,   297,   298,   299,   300,   301,   302,   303,   304,
     305,   306,   307,   308,   309,   310,   311,   312,   313,   314,
     315,   316,   317,   318,   319,   320,   321,   322,   323,   324,
     325,   326,   327,   328,   329,   330,   331,   332,   333,   334,
     335,   336,   337,   338,   339,   340,   341,   342,   343,   344,
     345,   346,   347,   348,   349,   350,   351,   352,   353,   354
};
# endif

/* YYR1[YYN] -- Symbol number of symbol that rule YYN derives.  */
static const yytype_uint8 yyr1[] =
{
       0,   100,   101,   101,   102,   102,   102,   102,   102,   102,
     102,   102,   102,   102,   102,   102,   102,   103,   104,   104,
     105,   105,   106,   106,   106,   106,   106,   106,   106,   106,
     106,   106,   106,   106,   106,   106,   106,   107,   107,   107,
     108,   108,   109,   109,   110,   110,   110,   110,   111,   111,
     112,   112,   113,   114,   114,   115,   115,   116,   116,   117,
     118,   118,   119,   119,   120,   120,   120,   120,   120,   120,
     120,   121,   122,   122,   123,   123,   124,   124,   124,   124,
     124,   124,   124,   124,   124,   124,   124,   124,   124,   125,
     125,   126,   126,   127,   127,   128,   128,   129,   129,   130,
     130,   131,   131,   131,   131,   131,   131,   131,   131,   131,
     131,   132,   132,   133,   133,   134,   134,   134,   134,   134,
     134,   134,   134,   134,   135,   135,   135,   135,   135,   135,
     135,   135,   136,   137,   137,   138,   138,   138,   138,   138,
     138,   138,   138,   138,   138,   138,   138,   138,   138,   138,
     138,   138,   138,   138,   138,   138,   138,   138,   138,   138,
     139,   139,   140,   140,   140,   140,   141,   141,   142,   143,
     144,   144,   145,   146,   146,   147,   147,   148,   149,   149,
     150,   151,   151,   152,   152,   153,   153,   153,   153,   153,
     153,   153,   153,   154,   154,   155,   155,   156,   157,   158,
     158,   158,   158,   159,   160,   161,   161,   162,   162,   163,
     163,   164,   164,   165,   165,   166,   166,   166,   166,   166,
     166,   166,   166,   166,   166,   166,   166,   166,   167,   167,
     168,   168,   169,   169,   170,   170,   171,   171,   172,   172,
     172,   173,   173,   173,   173,   173,   174,   174,   174,   174,
     174,   175,   175,   175,   176,   176,   176,   176,   177,   177,
     178,   178,   178,   178,   178,   179,   179,   179,   179,   179,
     179,   179,   180,   180,   180,   180,   181,   181,   182,   182,
     182,   182,   182,   182,   183,   183,   183,   183,   183,   183,
     183,   183,   183,   183,   183,   183,   183,   184,   184,   184,
     185,   185,   185,   186,   186,   187,   187,   188,   188,   189,
     189,   189,   190,   190,   191,   192,   192,   192,   192
};

/* YYR2[YYN] -- Number of symbols composing right hand side of rule YYN.  */
static const yytype_uint8 yyr2[] =
{
       0,     2,     1,     2,     2,     1,     1,     1,     1,     1,
       1,     1,     1,     1,     1,     1,     1,     6,     2,     3,
       1,     2,     2,     1,     1,     1,     1,     1,     1,     1,
       1,     1,     1,     1,     1,     1,     1,     0,     2,     2,
       0,     2,     0,     2,     1,     1,     3,     3,     5,     7,
       5,     8,     5,     5,     7,     5,     7,     5,     7,     4,
       2,     3,     1,     2,     2,     1,     1,     1,     1,     1,
       1,     4,     2,     3,     1,     2,     2,     1,     1,     1,
       1,     1,     1,     1,     1,     1,     1,     1,     1,     5,
       7,     5,     7,     5,     7,     2,     6,     2,     3,     1,
       2,     2,     1,     1,     1,     1,     1,     1,     1,     1,
       1,     2,     3,     1,     2,     2,     1,     1,     1,     1,
       1,     1,     1,     1,     4,     5,     8,     6,     6,     7,
      10,     8,     2,     4,     5,     2,     3,     1,     1,     1,
       1,     1,     1,     1,     1,     1,     1,     1,     1,     1,
       1,     1,     1,     1,     1,     1,     1,     1,     1,     1,
       1,     1,     5,     7,     6,     8,     1,     2,     5,     8,
       0,     2,     5,     3,     5,     1,     2,     5,     7,    11,
       8,     2,     3,     1,     2,     2,     1,     1,     1,     1,
       1,     1,     1,     0,     4,     2,     3,     2,     2,     5,
       6,     9,     7,     5,     2,     4,     5,     4,     5,     2,
       3,     1,     2,     1,     3,     1,     1,     1,     1,     1,
       1,     1,     1,     1,     1,     1,     1,     1,     1,     3,
       1,     3,     1,     3,     1,     3,     1,     3,     1,     3,
       3,     1,     3,     3,     3,     3,     1,     3,     3,     3,
       3,     1,     3,     3,     1,     3,     3,     3,     1,     3,
       1,     2,     2,     2,     2,     1,     4,     6,     7,     4,
       5,     3,     0,     2,     3,     9,     1,     2,     2,     1,
       1,     1,     1,     1,     1,     3,     1,     1,     1,     1,
       1,     1,     1,     1,     1,     1,     1,     2,     3,     2,
       2,     3,     4,     2,     3,     1,     3,     1,     3,     2,
       3,     6,     1,     2,     4,     5,     5,     5,     5
};

/* YYDEFACT[STATE-NAME] -- Default reduction number in state STATE-NUM.
   Performed when YYTABLE doesn't specify something else to do.  Zero
   means the default is an error.  */
static const yytype_uint16 yydefact[] =
{
       0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
       0,     2,    12,    13,    11,     5,     0,     7,   160,   161,
      10,    14,    15,     6,     8,     9,    16,     0,     0,    37,
      42,    40,     0,     0,     0,     0,     0,   287,   288,   289,
     286,   197,   198,   195,   293,     0,   290,   291,   292,     0,
       0,     0,     0,     0,     0,     0,     0,     4,   213,   228,
     230,   232,   234,   236,   238,   241,   246,   251,   254,   258,
     260,   265,   284,   294,     0,   295,   296,     1,     3,     0,
      95,     0,     0,     0,    42,     0,     0,     0,     0,     0,
       0,     0,     0,   209,   211,     0,     0,     0,   303,   196,
       0,     0,   309,     0,     0,   312,   300,   307,     0,   261,
     260,   264,   263,   262,   299,     0,     0,     0,     0,     0,
       0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
       0,     0,     0,     0,     0,     0,     0,     0,     0,   215,
     216,   217,   218,   219,   220,   221,   222,   223,   224,   225,
     226,   227,     0,   297,    97,     0,   104,     0,    99,   103,
     102,   105,   106,   108,   107,   109,   110,     0,   305,   124,
       0,     0,     0,    39,    38,    40,    44,    43,    45,     0,
      71,    41,     0,    59,     0,     0,     0,     0,     0,   210,
     212,     0,   272,   307,     0,     0,   285,     0,     0,   310,
       0,   313,     0,   301,     0,   298,   229,   231,   233,   235,
     237,   239,   240,   242,   243,   244,   245,   247,   248,   249,
     250,   252,   253,   255,   256,   257,   259,   271,     0,   214,
     304,     0,   204,   101,    98,   100,     0,     0,     0,   125,
       0,     0,     0,     0,    72,     0,    81,    85,    78,     0,
      74,    82,    83,    84,    79,    80,    77,    86,    87,    88,
      60,     0,     0,    62,    66,    65,    68,    69,    67,    70,
     162,   170,     0,     0,     0,   172,     0,     0,   269,   272,
       0,   203,     0,     0,     0,     0,   302,   308,     0,   266,
       0,   111,     0,   118,     0,   113,   117,   116,   119,   120,
     122,   121,   123,     0,   127,   306,     0,   128,     0,     0,
       0,    17,    46,    47,     0,     0,     0,    76,    73,    75,
       0,    64,    61,    63,     0,     0,   164,   166,     0,     0,
       0,     0,     0,     0,   175,     0,     0,   273,     0,   280,
     279,   281,   282,   283,     0,   276,   270,   315,   316,   314,
       0,   318,   317,   272,     0,   207,     0,     0,   115,   112,
     114,    96,     0,     0,   129,     0,     0,     0,     0,    18,
       0,     0,    20,    28,    29,    30,    31,    32,    33,    24,
      25,    26,     0,    27,    23,    34,    35,    36,     0,     0,
       0,     0,   163,     0,     0,   167,   171,     0,   178,     0,
       0,     0,   173,   176,     0,   193,     0,   278,   274,   277,
     311,   267,   272,   208,     0,   126,   131,     0,     0,   137,
     138,   139,   146,   147,   148,   149,   150,   151,   152,   153,
     154,   155,   156,   140,   141,   142,   143,   144,   145,   157,
     158,   159,     0,     0,     0,     0,     0,     0,     0,     0,
       0,    22,    19,    21,   132,     0,     0,     0,     0,     0,
       0,     0,     0,   165,   169,     0,     0,     0,   181,     0,
     187,   188,   189,   186,   190,   191,     0,   183,   192,     0,
     180,     0,   268,   205,     0,     0,   135,     0,     0,     0,
       0,     0,     0,     0,     0,     0,    93,     0,    91,     0,
      89,     0,   199,     0,     0,     0,     0,     0,   174,   185,
     182,   184,     0,     0,   206,   130,   136,   133,     0,     0,
       0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
       0,     0,     0,     0,   200,     0,   168,     0,   177,     0,
       0,     0,   134,     0,     0,    57,     0,    55,     0,    53,
       0,    48,    50,    52,    94,    92,    90,   202,     0,   179,
     194,     0,     0,     0,     0,     0,     0,     0,     0,    49,
       0,    58,    56,    54,   201,   275,    51
};

/* YYDEFGOTO[NTERM-NUM].  */
static const yytype_int16 yydefgoto[] =
{
      -1,    10,    11,    12,   311,   371,   372,    84,    88,    86,
     177,   373,   374,   375,   376,   377,   378,    13,   183,   262,
     263,    14,   180,   249,   250,   251,   252,   253,    15,    80,
     157,   158,   237,   294,   295,    16,   381,   382,   442,    17,
      18,   326,   327,    19,   329,    20,   275,   333,   334,    21,
      22,   405,   476,   477,   480,    23,    24,    25,   268,    26,
     165,   302,   166,    36,    95,   193,   152,    58,    59,    60,
      61,    62,    63,    64,    65,    66,    67,    68,    69,    70,
     278,   344,   345,    71,    72,    73,    74,   171,   108,    75,
     104,   105,    76
};

/* YYPACT[STATE-NUM] -- Index in YYTABLE of the portion describing
   STATE-NUM.  */
#define YYPACT_NINF -463
static const yytype_int16 yypact[] =
{
     449,    18,    35,    53,    76,    -8,    43,    46,   115,   317,
     116,  -463,  -463,  -463,  -463,  -463,   136,  -463,  -463,  -463,
    -463,  -463,  -463,  -463,  -463,  -463,  -463,    85,   157,   141,
     206,   213,  1075,    17,  1075,    38,   192,  -463,  -463,  -463,
     -26,  -463,  -463,  1075,  -463,   182,  -463,  -463,  -463,  1075,
     691,   757,  1075,  1075,  1075,  1075,   236,  -463,   187,   195,
     185,   198,   203,    37,   173,   120,   149,   172,   207,  -463,
    1160,  -463,  -463,  -463,   271,  -463,  -463,  -463,  -463,   199,
     243,    19,   274,    10,   206,    14,   227,    14,   231,   -10,
     262,   282,   244,  -463,  -463,   694,   256,   791,  -463,  -463,
    1075,     1,  -463,   215,   835,  -463,  -463,   217,     7,  -463,
       6,  -463,  -463,  -463,   202,   303,  1075,  1075,  1075,  1075,
    1075,  1075,  1075,  1075,  1075,  1075,  1075,  1075,  1075,  1075,
    1075,  1075,  1075,  1075,  1075,  1075,  1075,   310,  1075,  -463,
    -463,  -463,  -463,  -463,  -463,  -463,  -463,  -463,  -463,  -463,
    -463,  -463,  1075,   221,  -463,   525,  -463,   287,  -463,  -463,
    -463,  -463,  -463,  -463,  -463,  -463,  -463,   265,  -463,  -463,
     322,    -2,   278,   202,  -463,   213,   202,   273,  -463,   941,
    -463,   273,   551,  -463,   115,  1075,  1075,   275,   279,  -463,
    -463,   333,   289,  -463,    79,   288,  -463,  1075,  1075,  -463,
     250,  -463,  1075,  -463,   862,   221,   195,   185,   198,   203,
      37,   173,   173,   120,   120,   120,   120,   149,   149,   149,
     149,   172,   172,   207,   207,   207,  -463,   300,   304,  -463,
    -463,   313,  -463,  -463,  -463,  -463,   330,   326,   315,  -463,
      24,    23,   312,    15,  -463,   589,  -463,  -463,  -463,  1017,
    -463,  -463,  -463,  -463,  -463,  -463,  -463,  -463,  -463,  -463,
    -463,   686,   748,  -463,  -463,  -463,  -463,  -463,  -463,  -463,
     223,   311,   319,   374,   357,  -463,   331,   259,  -463,   289,
    1075,  -463,    65,   328,  1075,    98,  -463,  -463,   933,  -463,
     938,  -463,   615,  -463,   749,  -463,  -463,  -463,  -463,  -463,
    -463,  -463,  -463,   265,  -463,  -463,   383,  -463,   385,    97,
     395,  -463,   202,  -463,   337,   338,   340,  -463,  -463,  -463,
     394,  -463,  -463,  -463,   115,   349,   230,  -463,   399,   353,
     115,   355,   358,   122,  -463,   362,   321,  -463,  1004,  -463,
    -463,  -463,  -463,  -463,   826,  -463,  -463,  -463,  -463,  -463,
     109,  -463,  -463,   289,   101,  -463,   128,   369,  -463,  -463,
    -463,  -463,   356,   370,  -463,    25,  1091,   366,   368,  -463,
     499,  1087,  -463,  -463,  -463,  -463,  -463,  -463,  -463,  -463,
    -463,  -463,   136,  -463,  -463,  -463,  -463,  -463,    21,    22,
      26,   375,  -463,  1075,   115,  -463,  -463,   115,  -463,   404,
    1075,   115,  -463,  -463,  1205,   390,   380,  -463,  -463,  -463,
    -463,  -463,   289,  -463,  1009,  -463,  -463,   424,   384,  -463,
    -463,  -463,  -463,  -463,  -463,  -463,  -463,  -463,  -463,  -463,
    -463,  -463,  -463,  -463,  -463,  -463,  -463,  -463,  -463,  -463,
    -463,  -463,   379,   434,   439,   391,   392,   393,   396,   398,
     407,  -463,  -463,  -463,  -463,   389,   411,   408,   420,   410,
     427,    61,   415,  -463,  -463,  1075,   133,   418,  -463,  1004,
    -463,  -463,  -463,  -463,  -463,  -463,  1206,  -463,  -463,   397,
    -463,   469,  -463,  -463,   135,   423,   414,    12,   421,   422,
      34,    36,    47,   474,   475,   481,  -463,   487,  -463,   488,
    -463,   490,  -463,   491,   138,   115,   445,   115,  -463,  -463,
    -463,  -463,   108,    83,  -463,  -463,  -463,  -463,   153,   448,
     454,   451,   465,   453,   467,   455,   477,   462,   466,   468,
     470,   478,   479,   473,  -463,    33,  -463,   115,  -463,  1242,
     435,   516,  -463,   484,   532,  -463,   537,  -463,   538,  -463,
     539,  -463,  -463,  -463,  -463,  -463,  -463,  -463,   540,  -463,
    -463,   124,   115,   494,   492,   493,   495,   503,   837,  -463,
     115,  -463,  -463,  -463,  -463,  -463,  -463
};

/* YYPGOTO[NTERM-NUM].  */
static const yytype_int16 yypgoto[] =
{
    -463,  -463,    -1,   -71,  -463,  -463,   177,  -463,   387,   476,
     482,  -463,  -463,  -463,  -463,  -463,  -463,  -157,  -463,  -463,
     308,  -176,  -463,  -463,   324,  -463,  -463,  -463,  -175,   193,
    -463,   425,   277,  -463,   290,  -463,  -463,  -463,  -463,   -79,
    -463,  -463,   257,  -463,  -463,   -78,  -463,  -463,   252,   -77,
     -73,  -463,    74,  -462,  -463,   -74,  -463,  -463,  -463,   -69,
    -463,  -463,  -463,  -151,  -463,   240,  -463,  -463,   483,   480,
     489,   472,   496,     8,   180,   247,   137,   129,   -17,  1067,
    -266,    39,  -333,  -463,   -68,  -463,   545,  -234,   -85,  -463,
    -463,   498,  -463
};

/* YYTABLE[YYPACT[STATE-NUM]].  What to do in state STATE-NUM.  If
   positive, shift that token.  If negative, reduce the rule which
   number is the opposite.  If YYTABLE_NINF, syntax error.  */
#define YYTABLE_NINF -1
static const yytype_uint16 yytable[] =
{
     159,   160,   161,   248,   254,   163,   162,   309,   156,    78,
     164,   409,   194,   346,   511,   174,   173,   178,   168,   178,
     176,   312,   247,    90,    27,   168,    97,   455,   457,   168,
     305,   305,   459,   270,    94,   109,   111,   112,   113,   305,
     521,    29,   523,   184,    32,     1,     2,     3,     4,   185,
     137,   239,    28,   525,   196,   456,   458,   240,     5,    30,
     460,     6,   138,     7,   203,   517,   204,   168,   522,    91,
     524,    98,   169,   248,   254,     8,   307,   511,   159,   160,
     161,   526,    31,   163,   162,   170,   156,   411,   164,   308,
     306,   417,   247,    93,   190,    33,     9,   197,    34,   558,
     255,   256,   257,   264,   265,   266,   258,    56,   246,   267,
     259,    56,    56,   269,   502,     1,    77,     3,   347,   226,
     121,   122,   348,     1,     2,     3,     4,   503,     5,   211,
     212,     6,   279,     7,   379,   380,     5,    81,   280,     6,
     540,     7,   541,   401,     5,     8,   482,     6,   332,     7,
     364,   351,    83,     8,   412,   352,   365,   296,   297,   298,
     280,     8,   300,   299,   410,   293,   469,   301,   349,    35,
     255,   256,   257,   392,     9,   313,   258,   402,   246,   398,
     259,   413,   338,   264,   265,   266,   507,   280,   514,   267,
      79,   534,   280,   269,   280,   379,   380,   535,   339,   340,
     341,    82,   127,   354,   342,   356,   542,     2,   343,   128,
     129,   130,   541,   131,   132,   296,   297,   298,    85,     5,
     300,   299,     6,   293,     7,   301,    87,   504,   470,   471,
      96,   383,   384,   385,   100,   409,     8,   386,   133,   134,
     135,   387,   114,   463,   324,   325,   464,   513,   116,    57,
     467,   394,   325,   518,   154,   117,   118,   155,   123,   124,
     125,   126,   223,   224,   225,   339,   340,   341,   221,   222,
     119,   342,    89,   120,    92,   343,   136,   153,   167,     5,
     172,   179,     6,    99,     7,   182,   186,   336,   187,   101,
     103,   107,   383,   384,   385,     2,     8,   188,   386,    98,
     470,   471,   387,   213,   214,   215,   216,     5,   191,   205,
       6,   198,     7,   202,   337,   466,   227,   338,   230,   236,
      37,    38,    39,    40,     8,   472,   473,   474,   238,   484,
     241,   475,   243,   274,   273,   478,   470,   471,     2,   276,
     195,   281,   234,   277,   200,   155,   284,    41,    42,    43,
       5,    44,   288,     6,   536,     7,   538,    45,    46,    47,
      48,   289,   303,   470,   471,   290,   310,     8,   304,    49,
     328,    50,   330,    51,   217,   218,   219,   220,   228,    52,
     331,    53,    54,   332,   335,   291,   559,   349,   292,   362,
      55,   363,   229,   388,   389,   233,   390,   472,   473,   474,
     391,   393,   366,   475,     3,   396,   397,   478,   399,   415,
     400,   569,   367,   368,    56,     5,   404,   406,     6,   576,
       7,   414,   443,   416,   444,   271,   272,   461,   465,   479,
     485,   487,     8,   472,   473,   474,   481,   282,   283,   475,
     488,   486,   285,   478,   287,   489,   496,   490,   491,   492,
     369,   512,   493,   370,   494,   497,     1,     2,     3,     4,
     472,   473,   474,   495,   499,   498,   475,   500,   505,     5,
     478,   501,     6,   508,     7,   168,   515,   516,   519,   520,
     527,   528,   339,   340,   341,   317,     8,   529,   342,   339,
     340,   341,   343,   530,   531,   342,   532,   533,   537,   343,
     543,   321,    37,    38,    39,    40,   544,     9,   545,   546,
     547,   548,   549,   445,   446,   447,   448,   449,   450,   551,
     287,   550,   305,   552,   350,   553,   557,   554,    37,    38,
      39,    40,   358,    44,   561,   555,   556,   562,   563,    45,
      46,    47,    48,   564,   565,   566,   567,   570,   453,   571,
     572,    49,   573,    50,   231,    51,   574,    43,   232,    44,
     175,    52,   242,    53,    54,    45,    46,    47,    48,   181,
     323,     5,    55,   319,     6,   454,     7,    49,   407,    50,
     361,    51,   235,   395,   360,   403,   539,    52,     8,    53,
      54,   209,    37,    38,    39,    40,    56,   207,    55,   206,
     568,   115,   201,   314,   315,   316,   260,   208,     0,   261,
     451,     0,     0,     0,     0,     0,   210,     0,    37,    38,
      39,    40,    56,    44,     0,     0,     0,     0,     0,    45,
      46,    47,    48,   462,     0,     0,     0,     0,     0,     0,
       0,    49,   357,    50,     0,    51,     0,    43,     0,    44,
       0,    52,     0,    53,    54,    45,    46,    47,    48,     0,
       0,     0,    55,     0,     0,     0,     0,    49,     0,    50,
       0,    51,     0,     0,     0,     0,     0,    52,     0,    53,
      54,     0,     0,     0,     0,     0,    56,     0,    55,    37,
      38,    39,    40,   320,    37,    38,    39,    40,     0,     0,
       0,     1,     2,     3,     4,   506,     0,     0,     0,   509,
       0,     0,    56,     0,     5,     0,     0,     6,     0,     7,
      44,     0,     0,     0,     0,    44,    45,    46,    47,    48,
       0,     8,    46,    47,    48,     0,     0,     0,    49,     0,
      50,     0,    51,    49,     0,    50,   102,    51,    52,   189,
      53,    54,     9,    52,     0,    53,    54,     2,     0,    55,
      37,    38,    39,    40,    55,     0,     0,     0,     5,     5,
       0,     6,     6,     7,     7,     0,     0,     0,     0,     0,
       0,     0,     0,    56,     0,     8,     8,     0,    56,     0,
       0,    44,     0,     0,    37,    38,    39,    40,    46,    47,
      48,     0,     0,   322,   359,     0,   261,   292,     0,    49,
       0,    50,     0,    51,   106,     0,     0,     0,     0,    52,
       0,    53,    54,     0,     0,    44,     0,     0,     0,     0,
      55,     0,    46,    47,    48,     0,     0,     0,    37,    38,
      39,    40,     0,    49,   192,    50,     5,    51,     0,     6,
       0,     7,     0,    52,    56,    53,    54,     5,     0,     0,
       6,     0,     7,     8,    55,    37,    38,    39,    40,    44,
       0,     0,     0,     0,     8,     0,    46,    47,    48,     0,
       0,   408,     0,     0,   338,     0,     0,    49,    56,    50,
     199,    51,   575,     0,     0,   338,    44,    52,     0,    53,
      54,     0,     0,    46,    47,    48,     0,     0,    55,     0,
       0,     0,     0,     0,    49,     0,    50,     0,    51,   286,
       0,     0,     0,     0,    52,     0,    53,    54,     0,     0,
       0,     0,    56,     0,     0,    55,    37,    38,    39,    40,
       0,    37,    38,    39,    40,     0,     0,     0,     1,     2,
       3,     4,     0,     0,     0,     0,     0,     0,     0,    56,
       0,     5,     0,     0,     6,     0,     7,    44,     0,     0,
       0,     0,    44,     0,    46,    47,    48,     0,     8,    46,
      47,    48,     0,     0,     0,    49,   353,    50,     0,    51,
      49,   355,    50,     0,    51,    52,   244,    53,    54,   245,
      52,     0,    53,    54,     0,     0,    55,    37,    38,    39,
      40,    55,    37,    38,    39,    40,     0,     0,     0,     0,
       0,     0,     0,     0,     1,     2,     3,     4,     0,     0,
      56,     0,     0,     0,     0,    56,     0,     5,    44,     0,
       6,     0,     7,    44,    45,    46,    47,    48,     0,     0,
      46,    47,    48,     0,     8,     0,    49,     0,    50,     0,
      51,    49,   483,    50,     0,    51,    52,     0,    53,    54,
       0,    52,   318,    53,    54,   245,     0,    55,    37,    38,
      39,    40,    55,     0,     0,     0,     0,     0,     0,     0,
       0,     0,     0,     0,   366,     0,     3,    27,     0,     0,
       0,    56,     0,     0,   367,   368,    56,     5,     0,    44,
       6,     0,     7,     0,     0,     0,    46,    47,    48,   110,
     110,   110,   110,     0,     8,    28,     0,    49,     0,    50,
       0,    51,     0,     0,     0,     0,     0,    52,     0,    53,
      54,     0,   452,     0,     0,   370,     0,   418,    55,     0,
       0,   419,   420,   421,     0,   422,   423,   424,   425,   426,
     427,   428,   429,   430,   431,     0,     0,     0,     0,     0,
       0,     0,    56,   432,   433,   434,   435,   436,   437,   438,
     439,   440,   441,   110,   110,   110,   110,   110,   110,   110,
     110,   110,   110,   110,   110,   110,   110,   110,   110,   110,
     110,   110,   110,   110,   137,     0,     0,     0,     0,     0,
       0,     0,     1,     1,     3,     3,   138,     0,     0,     0,
       0,     0,     0,   139,     0,     5,     5,     0,     6,     6,
       7,     7,     0,     0,   140,   141,   142,   143,   144,   145,
     146,   147,     8,     8,     0,     0,     0,     0,     0,     1,
       0,     3,   148,   149,   150,   151,     0,     0,     0,     0,
     468,   510,     5,   469,   469,     6,     0,     7,     0,     0,
       0,     0,     0,     0,     0,     0,     0,     0,     0,     8,
       0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
       0,     0,     0,     0,     0,     0,     0,   560,     0,     0,
     469
};

#define yypact_value_is_default(Yystate) \
  (!!((Yystate) == (-463)))

#define yytable_value_is_error(Yytable_value) \
  YYID (0)

static const yytype_int16 yycheck[] =
{
      79,    79,    79,   179,   179,    79,    79,   241,    79,    10,
      79,   344,    97,   279,   476,    83,     6,    85,     6,    87,
       6,     6,   179,     6,     6,     6,    52,     6,     6,     6,
       6,     6,     6,   184,    35,    52,    53,    54,    55,     6,
       6,     6,     6,    53,    52,     7,     8,     9,    10,    59,
      44,    53,    34,     6,    53,    34,    34,    59,    20,     6,
      34,    23,    56,    25,    57,    53,    59,     6,    34,    52,
      34,    97,    53,   249,   249,    37,    53,   539,   157,   157,
     157,    34,     6,   157,   157,    66,   157,   353,   157,    66,
      66,    66,   249,    55,    95,    52,    58,    96,    52,    66,
     179,   179,   179,   182,   182,   182,   179,    97,   179,   182,
     179,    97,    97,   182,    53,     7,     0,     9,    53,   136,
      83,    84,    57,     7,     8,     9,    10,    66,    20,   121,
     122,    23,    53,    25,   310,   310,    20,    52,    59,    23,
      57,    25,    59,    21,    20,    37,   412,    23,    26,    25,
      53,    53,    11,    37,    53,    57,    59,   236,   236,   236,
      59,    37,   236,   236,    55,   236,    58,   236,    59,    54,
     249,   249,   249,   324,    58,   243,   249,    55,   249,   330,
     249,    53,    58,   262,   262,   262,    53,    59,    53,   262,
      54,    53,    59,   262,    59,   371,   371,    59,   277,   277,
     277,    44,    82,   288,   277,   290,    53,     8,   277,    89,
      90,    91,    59,    64,    65,   294,   294,   294,    12,    20,
     294,   294,    23,   294,    25,   294,    13,   461,   404,   404,
      38,   310,   310,   310,    52,   568,    37,   310,    66,    67,
      68,   310,     6,   394,    21,    22,   397,   481,    61,     9,
     401,    21,    22,   487,    55,    60,    71,    58,    85,    86,
      87,    88,   133,   134,   135,   344,   344,   344,   131,   132,
      72,   344,    32,    70,    34,   344,    69,     6,    35,    20,
       6,    54,    23,    43,    25,    54,    24,    28,     6,    49,
      50,    51,   371,   371,   371,     8,    37,    53,   371,    97,
     476,   476,   371,   123,   124,   125,   126,    20,    52,     6,
      23,    96,    25,    96,    55,   400,     6,    58,    97,    54,
       3,     4,     5,     6,    37,   404,   404,   404,     6,   414,
      52,   404,    59,    54,    59,   404,   512,   512,     8,     6,
     100,    53,    55,    54,   104,    58,    96,    30,    31,    32,
      20,    34,    52,    23,   505,    25,   507,    40,    41,    42,
      43,    57,    36,   539,   539,    52,    54,    37,    53,    52,
      59,    54,    53,    56,   127,   128,   129,   130,   138,    62,
       6,    64,    65,    26,    53,    55,   537,    59,    58,     6,
      73,     6,   152,    56,    56,   155,    56,   476,   476,   476,
       6,    52,     7,   476,     9,     6,    53,   476,    53,    53,
      52,   562,    17,    18,    97,    20,    54,    96,    23,   570,
      25,    52,    56,    53,    56,   185,   186,    52,    24,    39,
       6,    52,    37,   512,   512,   512,    56,   197,   198,   512,
       6,    57,   202,   512,   204,     6,    57,    56,    56,    56,
      55,    54,    56,    58,    56,    44,     7,     8,     9,    10,
     539,   539,   539,    56,    44,    57,   539,    57,    53,    20,
     539,    44,    23,    55,    25,     6,    53,    63,    57,    57,
       6,     6,   561,   561,   561,   245,    37,     6,   561,   568,
     568,   568,   561,     6,     6,   568,     6,     6,    53,   568,
      52,   261,     3,     4,     5,     6,    52,    58,    57,    44,
      57,    44,    57,    14,    15,    16,    17,    18,    19,    57,
     280,    44,     6,    57,   284,    57,    53,    57,     3,     4,
       5,     6,   292,    34,    99,    57,    57,    53,     6,    40,
      41,    42,    43,     6,     6,     6,     6,    53,   371,    57,
      57,    52,    57,    54,    29,    56,    53,    32,    33,    34,
      84,    62,   175,    64,    65,    40,    41,    42,    43,    87,
     262,    20,    73,   249,    23,   382,    25,    52,   338,    54,
     303,    56,   157,   326,   294,   333,   512,    62,    37,    64,
      65,   119,     3,     4,     5,     6,    97,   117,    73,   116,
     561,    56,   104,    14,    15,    16,    55,   118,    -1,    58,
     370,    -1,    -1,    -1,    -1,    -1,   120,    -1,     3,     4,
       5,     6,    97,    34,    -1,    -1,    -1,    -1,    -1,    40,
      41,    42,    43,   393,    -1,    -1,    -1,    -1,    -1,    -1,
      -1,    52,    27,    54,    -1,    56,    -1,    32,    -1,    34,
      -1,    62,    -1,    64,    65,    40,    41,    42,    43,    -1,
      -1,    -1,    73,    -1,    -1,    -1,    -1,    52,    -1,    54,
      -1,    56,    -1,    -1,    -1,    -1,    -1,    62,    -1,    64,
      65,    -1,    -1,    -1,    -1,    -1,    97,    -1,    73,     3,
       4,     5,     6,     7,     3,     4,     5,     6,    -1,    -1,
      -1,     7,     8,     9,    10,   465,    -1,    -1,    -1,   469,
      -1,    -1,    97,    -1,    20,    -1,    -1,    23,    -1,    25,
      34,    -1,    -1,    -1,    -1,    34,    40,    41,    42,    43,
      -1,    37,    41,    42,    43,    -1,    -1,    -1,    52,    -1,
      54,    -1,    56,    52,    -1,    54,    55,    56,    62,    55,
      64,    65,    58,    62,    -1,    64,    65,     8,    -1,    73,
       3,     4,     5,     6,    73,    -1,    -1,    -1,    20,    20,
      -1,    23,    23,    25,    25,    -1,    -1,    -1,    -1,    -1,
      -1,    -1,    -1,    97,    -1,    37,    37,    -1,    97,    -1,
      -1,    34,    -1,    -1,     3,     4,     5,     6,    41,    42,
      43,    -1,    -1,    55,    55,    -1,    58,    58,    -1,    52,
      -1,    54,    -1,    56,    57,    -1,    -1,    -1,    -1,    62,
      -1,    64,    65,    -1,    -1,    34,    -1,    -1,    -1,    -1,
      73,    -1,    41,    42,    43,    -1,    -1,    -1,     3,     4,
       5,     6,    -1,    52,    53,    54,    20,    56,    -1,    23,
      -1,    25,    -1,    62,    97,    64,    65,    20,    -1,    -1,
      23,    -1,    25,    37,    73,     3,     4,     5,     6,    34,
      -1,    -1,    -1,    -1,    37,    -1,    41,    42,    43,    -1,
      -1,    55,    -1,    -1,    58,    -1,    -1,    52,    97,    54,
      55,    56,    55,    -1,    -1,    58,    34,    62,    -1,    64,
      65,    -1,    -1,    41,    42,    43,    -1,    -1,    73,    -1,
      -1,    -1,    -1,    -1,    52,    -1,    54,    -1,    56,    57,
      -1,    -1,    -1,    -1,    62,    -1,    64,    65,    -1,    -1,
      -1,    -1,    97,    -1,    -1,    73,     3,     4,     5,     6,
      -1,     3,     4,     5,     6,    -1,    -1,    -1,     7,     8,
       9,    10,    -1,    -1,    -1,    -1,    -1,    -1,    -1,    97,
      -1,    20,    -1,    -1,    23,    -1,    25,    34,    -1,    -1,
      -1,    -1,    34,    -1,    41,    42,    43,    -1,    37,    41,
      42,    43,    -1,    -1,    -1,    52,    53,    54,    -1,    56,
      52,    53,    54,    -1,    56,    62,    55,    64,    65,    58,
      62,    -1,    64,    65,    -1,    -1,    73,     3,     4,     5,
       6,    73,     3,     4,     5,     6,    -1,    -1,    -1,    -1,
      -1,    -1,    -1,    -1,     7,     8,     9,    10,    -1,    -1,
      97,    -1,    -1,    -1,    -1,    97,    -1,    20,    34,    -1,
      23,    -1,    25,    34,    40,    41,    42,    43,    -1,    -1,
      41,    42,    43,    -1,    37,    -1,    52,    -1,    54,    -1,
      56,    52,    53,    54,    -1,    56,    62,    -1,    64,    65,
      -1,    62,    55,    64,    65,    58,    -1,    73,     3,     4,
       5,     6,    73,    -1,    -1,    -1,    -1,    -1,    -1,    -1,
      -1,    -1,    -1,    -1,     7,    -1,     9,     6,    -1,    -1,
      -1,    97,    -1,    -1,    17,    18,    97,    20,    -1,    34,
      23,    -1,    25,    -1,    -1,    -1,    41,    42,    43,    52,
      53,    54,    55,    -1,    37,    34,    -1,    52,    -1,    54,
      -1,    56,    -1,    -1,    -1,    -1,    -1,    62,    -1,    64,
      65,    -1,    55,    -1,    -1,    58,    -1,    56,    73,    -1,
      -1,    60,    61,    62,    -1,    64,    65,    66,    67,    68,
      69,    70,    71,    72,    73,    -1,    -1,    -1,    -1,    -1,
      -1,    -1,    97,    82,    83,    84,    85,    86,    87,    88,
      89,    90,    91,   116,   117,   118,   119,   120,   121,   122,
     123,   124,   125,   126,   127,   128,   129,   130,   131,   132,
     133,   134,   135,   136,    44,    -1,    -1,    -1,    -1,    -1,
      -1,    -1,     7,     7,     9,     9,    56,    -1,    -1,    -1,
      -1,    -1,    -1,    63,    -1,    20,    20,    -1,    23,    23,
      25,    25,    -1,    -1,    74,    75,    76,    77,    78,    79,
      80,    81,    37,    37,    -1,    -1,    -1,    -1,    -1,     7,
      -1,     9,    92,    93,    94,    95,    -1,    -1,    -1,    -1,
      55,    55,    20,    58,    58,    23,    -1,    25,    -1,    -1,
      -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,    37,
      -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,
      -1,    -1,    -1,    -1,    -1,    -1,    -1,    55,    -1,    -1,
      58
};

/* YYSTOS[STATE-NUM] -- The (internal number of the) accessing
   symbol of state STATE-NUM.  */
static const yytype_uint8 yystos[] =
{
       0,     7,     8,     9,    10,    20,    23,    25,    37,    58,
     101,   102,   103,   117,   121,   128,   135,   139,   140,   143,
     145,   149,   150,   155,   156,   157,   159,     6,    34,     6,
       6,     6,    52,    52,    52,    54,   163,     3,     4,     5,
       6,    30,    31,    32,    34,    40,    41,    42,    43,    52,
      54,    56,    62,    64,    65,    73,    97,   165,   167,   168,
     169,   170,   171,   172,   173,   174,   175,   176,   177,   178,
     179,   183,   184,   185,   186,   189,   192,     0,   102,    54,
     129,    52,    44,    11,   107,    12,   109,    13,   108,   165,
       6,    52,   165,    55,   102,   164,    38,    52,    97,   165,
      52,   165,    55,   165,   190,   191,    57,   165,   188,   178,
     179,   178,   178,   178,     6,   186,    61,    60,    71,    72,
      70,    83,    84,    85,    86,    87,    88,    82,    89,    90,
      91,    64,    65,    66,    67,    68,    69,    44,    56,    63,
      74,    75,    76,    77,    78,    79,    80,    81,    92,    93,
      94,    95,   166,     6,    55,    58,   103,   130,   131,   139,
     145,   149,   150,   155,   159,   160,   162,    35,     6,    53,
      66,   187,     6,     6,   184,   109,     6,   110,   184,    54,
     122,   110,    54,   118,    53,    59,    24,     6,    53,    55,
     102,    52,    53,   165,   188,   165,    53,    96,    96,    55,
     165,   191,    96,    57,    59,     6,   168,   169,   170,   171,
     172,   173,   173,   174,   174,   174,   174,   175,   175,   175,
     175,   176,   176,   177,   177,   177,   178,     6,   165,   165,
      97,    29,    33,   165,    55,   131,    54,   132,     6,    53,
      59,    52,   108,    59,    55,    58,   103,   117,   121,   123,
     124,   125,   126,   127,   128,   139,   145,   149,   150,   159,
      55,    58,   119,   120,   139,   145,   149,   150,   158,   159,
     163,   165,   165,    59,    54,   146,     6,    54,   180,    53,
      59,    53,   165,   165,    96,   165,    57,   165,    52,    57,
      52,    55,    58,   103,   133,   134,   139,   145,   149,   150,
     155,   159,   161,    36,    53,     6,    66,    53,    66,   187,
      54,   104,     6,   184,    14,    15,    16,   165,    55,   124,
       7,   165,    55,   120,    21,    22,   141,   142,    59,   144,
      53,     6,    26,   147,   148,    53,    28,    55,    58,   139,
     145,   149,   150,   159,   181,   182,   180,    53,    57,    59,
     165,    53,    57,    53,   188,    53,   188,    27,   165,    55,
     134,   132,     6,     6,    53,    59,     7,    17,    18,    55,
      58,   105,   106,   111,   112,   113,   114,   115,   116,   121,
     128,   136,   137,   139,   145,   149,   150,   159,    56,    56,
      56,     6,   163,    52,    21,   142,     6,    53,   163,    53,
      52,    21,    55,   148,    54,   151,    96,   165,    55,   182,
      55,   180,    53,    53,    52,    53,    53,    66,    56,    60,
      61,    62,    64,    65,    66,    67,    68,    69,    70,    71,
      72,    73,    82,    83,    84,    85,    86,    87,    88,    89,
      90,    91,   138,    56,    56,    14,    15,    16,    17,    18,
      19,   165,    55,   106,   129,     6,    34,     6,    34,     6,
      34,    52,   165,   163,   163,    24,   188,   163,    55,    58,
     121,   128,   139,   145,   149,   150,   152,   153,   159,    39,
     154,    56,   180,    53,   188,     6,    57,    52,     6,     6,
      56,    56,    56,    56,    56,    56,    57,    44,    57,    44,
      57,    44,    53,    66,   187,    53,   165,    53,    55,   165,
      55,   153,    54,   187,    53,    53,    63,    53,   187,    57,
      57,     6,    34,     6,    34,     6,    34,     6,     6,     6,
       6,     6,     6,     6,    53,    59,   163,    53,   163,   152,
      57,    59,    53,    52,    52,    57,    44,    57,    44,    57,
      44,    57,    57,    57,    57,    57,    57,    53,    66,   163,
      55,    99,    53,     6,     6,     6,     6,     6,   181,   163,
      53,    57,    57,    57,    53,    55,   163
};

#define yyerrok		(yyerrstatus = 0)
#define yyclearin	(yychar = YYEMPTY)
#define YYEMPTY		(-2)
#define YYEOF		0

#define YYACCEPT	goto yyacceptlab
#define YYABORT		goto yyabortlab
#define YYERROR		goto yyerrorlab


/* Like YYERROR except do call yyerror.  This remains here temporarily
   to ease the transition to the new meaning of YYERROR, for GCC.
   Once GCC version 2 has supplanted version 1, this can go.  However,
   YYFAIL appears to be in use.  Nevertheless, it is formally deprecated
   in Bison 2.4.2's NEWS entry, where a plan to phase it out is
   discussed.  */

#define YYFAIL		goto yyerrlab
#if defined YYFAIL
  /* This is here to suppress warnings from the GCC cpp's
     -Wunused-macros.  Normally we don't worry about that warning, but
     some users do, and we want to make it easy for users to remove
     YYFAIL uses, which will produce warnings from Bison 2.5.  */
#endif

#define YYRECOVERING()  (!!yyerrstatus)

#define YYBACKUP(Token, Value)                                  \
do                                                              \
  if (yychar == YYEMPTY)                                        \
    {                                                           \
      yychar = (Token);                                         \
      yylval = (Value);                                         \
      YYPOPSTACK (yylen);                                       \
      yystate = *yyssp;                                         \
      goto yybackup;                                            \
    }                                                           \
  else                                                          \
    {                                                           \
      yyerror (YY_("syntax error: cannot back up")); \
      YYERROR;							\
    }								\
while (YYID (0))

/* Error token number */
#define YYTERROR	1
#define YYERRCODE	256


/* This macro is provided for backward compatibility. */
#ifndef YY_LOCATION_PRINT
# define YY_LOCATION_PRINT(File, Loc) ((void) 0)
#endif


/* YYLEX -- calling `yylex' with the right arguments.  */
#ifdef YYLEX_PARAM
# define YYLEX yylex (YYLEX_PARAM)
#else
# define YYLEX yylex ()
#endif

/* Enable debugging if requested.  */
#if YYDEBUG

# ifndef YYFPRINTF
#  include <stdio.h> /* INFRINGES ON USER NAME SPACE */
#  define YYFPRINTF fprintf
# endif

# define YYDPRINTF(Args)			\
do {						\
  if (yydebug)					\
    YYFPRINTF Args;				\
} while (YYID (0))

# define YY_SYMBOL_PRINT(Title, Type, Value, Location)			  \
do {									  \
  if (yydebug)								  \
    {									  \
      YYFPRINTF (stderr, "%s ", Title);					  \
      yy_symbol_print (stderr,						  \
		  Type, Value); \
      YYFPRINTF (stderr, "\n");						  \
    }									  \
} while (YYID (0))


/*--------------------------------.
| Print this symbol on YYOUTPUT.  |
`--------------------------------*/

/*ARGSUSED*/
#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
static void
yy_symbol_value_print (FILE *yyoutput, int yytype, YYSTYPE const * const yyvaluep)
#else
static void
yy_symbol_value_print (yyoutput, yytype, yyvaluep)
    FILE *yyoutput;
    int yytype;
    YYSTYPE const * const yyvaluep;
#endif
{
  FILE *yyo = yyoutput;
  YYUSE (yyo);
  if (!yyvaluep)
    return;
# ifdef YYPRINT
  if (yytype < YYNTOKENS)
    YYPRINT (yyoutput, yytoknum[yytype], *yyvaluep);
# else
  YYUSE (yyoutput);
# endif
  switch (yytype)
    {
      default:
        break;
    }
}


/*--------------------------------.
| Print this symbol on YYOUTPUT.  |
`--------------------------------*/

#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
static void
yy_symbol_print (FILE *yyoutput, int yytype, YYSTYPE const * const yyvaluep)
#else
static void
yy_symbol_print (yyoutput, yytype, yyvaluep)
    FILE *yyoutput;
    int yytype;
    YYSTYPE const * const yyvaluep;
#endif
{
  if (yytype < YYNTOKENS)
    YYFPRINTF (yyoutput, "token %s (", yytname[yytype]);
  else
    YYFPRINTF (yyoutput, "nterm %s (", yytname[yytype]);

  yy_symbol_value_print (yyoutput, yytype, yyvaluep);
  YYFPRINTF (yyoutput, ")");
}

/*------------------------------------------------------------------.
| yy_stack_print -- Print the state stack from its BOTTOM up to its |
| TOP (included).                                                   |
`------------------------------------------------------------------*/

#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
static void
yy_stack_print (yytype_int16 *yybottom, yytype_int16 *yytop)
#else
static void
yy_stack_print (yybottom, yytop)
    yytype_int16 *yybottom;
    yytype_int16 *yytop;
#endif
{
  YYFPRINTF (stderr, "Stack now");
  for (; yybottom <= yytop; yybottom++)
    {
      int yybot = *yybottom;
      YYFPRINTF (stderr, " %d", yybot);
    }
  YYFPRINTF (stderr, "\n");
}

# define YY_STACK_PRINT(Bottom, Top)				\
do {								\
  if (yydebug)							\
    yy_stack_print ((Bottom), (Top));				\
} while (YYID (0))


/*------------------------------------------------.
| Report that the YYRULE is going to be reduced.  |
`------------------------------------------------*/

#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
static void
yy_reduce_print (YYSTYPE *yyvsp, int yyrule)
#else
static void
yy_reduce_print (yyvsp, yyrule)
    YYSTYPE *yyvsp;
    int yyrule;
#endif
{
  int yynrhs = yyr2[yyrule];
  int yyi;
  unsigned long int yylno = yyrline[yyrule];
  YYFPRINTF (stderr, "Reducing stack by rule %d (line %lu):\n",
	     yyrule - 1, yylno);
  /* The symbols being reduced.  */
  for (yyi = 0; yyi < yynrhs; yyi++)
    {
      YYFPRINTF (stderr, "   $%d = ", yyi + 1);
      yy_symbol_print (stderr, yyrhs[yyprhs[yyrule] + yyi],
		       &(yyvsp[(yyi + 1) - (yynrhs)])
		       		       );
      YYFPRINTF (stderr, "\n");
    }
}

# define YY_REDUCE_PRINT(Rule)		\
do {					\
  if (yydebug)				\
    yy_reduce_print (yyvsp, Rule); \
} while (YYID (0))

/* Nonzero means print parse trace.  It is left uninitialized so that
   multiple parsers can coexist.  */
int yydebug;
#else /* !YYDEBUG */
# define YYDPRINTF(Args)
# define YY_SYMBOL_PRINT(Title, Type, Value, Location)
# define YY_STACK_PRINT(Bottom, Top)
# define YY_REDUCE_PRINT(Rule)
#endif /* !YYDEBUG */


/* YYINITDEPTH -- initial size of the parser's stacks.  */
#ifndef	YYINITDEPTH
# define YYINITDEPTH 200
#endif

/* YYMAXDEPTH -- maximum size the stacks can grow to (effective only
   if the built-in stack extension method is used).

   Do not make this value too large; the results are undefined if
   YYSTACK_ALLOC_MAXIMUM < YYSTACK_BYTES (YYMAXDEPTH)
   evaluated with infinite-precision integer arithmetic.  */

#ifndef YYMAXDEPTH
# define YYMAXDEPTH 10000
#endif


#if YYERROR_VERBOSE

# ifndef yystrlen
#  if defined __GLIBC__ && defined _STRING_H
#   define yystrlen strlen
#  else
/* Return the length of YYSTR.  */
#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
static YYSIZE_T
yystrlen (const char *yystr)
#else
static YYSIZE_T
yystrlen (yystr)
    const char *yystr;
#endif
{
  YYSIZE_T yylen;
  for (yylen = 0; yystr[yylen]; yylen++)
    continue;
  return yylen;
}
#  endif
# endif

# ifndef yystpcpy
#  if defined __GLIBC__ && defined _STRING_H && defined _GNU_SOURCE
#   define yystpcpy stpcpy
#  else
/* Copy YYSRC to YYDEST, returning the address of the terminating '\0' in
   YYDEST.  */
#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
static char *
yystpcpy (char *yydest, const char *yysrc)
#else
static char *
yystpcpy (yydest, yysrc)
    char *yydest;
    const char *yysrc;
#endif
{
  char *yyd = yydest;
  const char *yys = yysrc;

  while ((*yyd++ = *yys++) != '\0')
    continue;

  return yyd - 1;
}
#  endif
# endif

# ifndef yytnamerr
/* Copy to YYRES the contents of YYSTR after stripping away unnecessary
   quotes and backslashes, so that it's suitable for yyerror.  The
   heuristic is that double-quoting is unnecessary unless the string
   contains an apostrophe, a comma, or backslash (other than
   backslash-backslash).  YYSTR is taken from yytname.  If YYRES is
   null, do not copy; instead, return the length of what the result
   would have been.  */
static YYSIZE_T
yytnamerr (char *yyres, const char *yystr)
{
  if (*yystr == '"')
    {
      YYSIZE_T yyn = 0;
      char const *yyp = yystr;

      for (;;)
	switch (*++yyp)
	  {
	  case '\'':
	  case ',':
	    goto do_not_strip_quotes;

	  case '\\':
	    if (*++yyp != '\\')
	      goto do_not_strip_quotes;
	    /* Fall through.  */
	  default:
	    if (yyres)
	      yyres[yyn] = *yyp;
	    yyn++;
	    break;

	  case '"':
	    if (yyres)
	      yyres[yyn] = '\0';
	    return yyn;
	  }
    do_not_strip_quotes: ;
    }

  if (! yyres)
    return yystrlen (yystr);

  return yystpcpy (yyres, yystr) - yyres;
}
# endif

/* Copy into *YYMSG, which is of size *YYMSG_ALLOC, an error message
   about the unexpected token YYTOKEN for the state stack whose top is
   YYSSP.

   Return 0 if *YYMSG was successfully written.  Return 1 if *YYMSG is
   not large enough to hold the message.  In that case, also set
   *YYMSG_ALLOC to the required number of bytes.  Return 2 if the
   required number of bytes is too large to store.  */
static int
yysyntax_error (YYSIZE_T *yymsg_alloc, char **yymsg,
                yytype_int16 *yyssp, int yytoken)
{
  YYSIZE_T yysize0 = yytnamerr (YY_NULL, yytname[yytoken]);
  YYSIZE_T yysize = yysize0;
  enum { YYERROR_VERBOSE_ARGS_MAXIMUM = 5 };
  /* Internationalized format string. */
  const char *yyformat = YY_NULL;
  /* Arguments of yyformat. */
  char const *yyarg[YYERROR_VERBOSE_ARGS_MAXIMUM];
  /* Number of reported tokens (one for the "unexpected", one per
     "expected"). */
  int yycount = 0;

  /* There are many possibilities here to consider:
     - Assume YYFAIL is not used.  It's too flawed to consider.  See
       <http://lists.gnu.org/archive/html/bison-patches/2009-12/msg00024.html>
       for details.  YYERROR is fine as it does not invoke this
       function.
     - If this state is a consistent state with a default action, then
       the only way this function was invoked is if the default action
       is an error action.  In that case, don't check for expected
       tokens because there are none.
     - The only way there can be no lookahead present (in yychar) is if
       this state is a consistent state with a default action.  Thus,
       detecting the absence of a lookahead is sufficient to determine
       that there is no unexpected or expected token to report.  In that
       case, just report a simple "syntax error".
     - Don't assume there isn't a lookahead just because this state is a
       consistent state with a default action.  There might have been a
       previous inconsistent state, consistent state with a non-default
       action, or user semantic action that manipulated yychar.
     - Of course, the expected token list depends on states to have
       correct lookahead information, and it depends on the parser not
       to perform extra reductions after fetching a lookahead from the
       scanner and before detecting a syntax error.  Thus, state merging
       (from LALR or IELR) and default reductions corrupt the expected
       token list.  However, the list is correct for canonical LR with
       one exception: it will still contain any token that will not be
       accepted due to an error action in a later state.
  */
  if (yytoken != YYEMPTY)
    {
      int yyn = yypact[*yyssp];
      yyarg[yycount++] = yytname[yytoken];
      if (!yypact_value_is_default (yyn))
        {
          /* Start YYX at -YYN if negative to avoid negative indexes in
             YYCHECK.  In other words, skip the first -YYN actions for
             this state because they are default actions.  */
          int yyxbegin = yyn < 0 ? -yyn : 0;
          /* Stay within bounds of both yycheck and yytname.  */
          int yychecklim = YYLAST - yyn + 1;
          int yyxend = yychecklim < YYNTOKENS ? yychecklim : YYNTOKENS;
          int yyx;

          for (yyx = yyxbegin; yyx < yyxend; ++yyx)
            if (yycheck[yyx + yyn] == yyx && yyx != YYTERROR
                && !yytable_value_is_error (yytable[yyx + yyn]))
              {
                if (yycount == YYERROR_VERBOSE_ARGS_MAXIMUM)
                  {
                    yycount = 1;
                    yysize = yysize0;
                    break;
                  }
                yyarg[yycount++] = yytname[yyx];
                {
                  YYSIZE_T yysize1 = yysize + yytnamerr (YY_NULL, yytname[yyx]);
                  if (! (yysize <= yysize1
                         && yysize1 <= YYSTACK_ALLOC_MAXIMUM))
                    return 2;
                  yysize = yysize1;
                }
              }
        }
    }

  switch (yycount)
    {
# define YYCASE_(N, S)                      \
      case N:                               \
        yyformat = S;                       \
      break
      YYCASE_(0, YY_("syntax error"));
      YYCASE_(1, YY_("syntax error, unexpected %s"));
      YYCASE_(2, YY_("syntax error, unexpected %s, expecting %s"));
      YYCASE_(3, YY_("syntax error, unexpected %s, expecting %s or %s"));
      YYCASE_(4, YY_("syntax error, unexpected %s, expecting %s or %s or %s"));
      YYCASE_(5, YY_("syntax error, unexpected %s, expecting %s or %s or %s or %s"));
# undef YYCASE_
    }

  {
    YYSIZE_T yysize1 = yysize + yystrlen (yyformat);
    if (! (yysize <= yysize1 && yysize1 <= YYSTACK_ALLOC_MAXIMUM))
      return 2;
    yysize = yysize1;
  }

  if (*yymsg_alloc < yysize)
    {
      *yymsg_alloc = 2 * yysize;
      if (! (yysize <= *yymsg_alloc
             && *yymsg_alloc <= YYSTACK_ALLOC_MAXIMUM))
        *yymsg_alloc = YYSTACK_ALLOC_MAXIMUM;
      return 1;
    }

  /* Avoid sprintf, as that infringes on the user's name space.
     Don't have undefined behavior even if the translation
     produced a string with the wrong number of "%s"s.  */
  {
    char *yyp = *yymsg;
    int yyi = 0;
    while ((*yyp = *yyformat) != '\0')
      if (*yyp == '%' && yyformat[1] == 's' && yyi < yycount)
        {
          yyp += yytnamerr (yyp, yyarg[yyi++]);
          yyformat += 2;
        }
      else
        {
          yyp++;
          yyformat++;
        }
  }
  return 0;
}
#endif /* YYERROR_VERBOSE */

/*-----------------------------------------------.
| Release the memory associated to this symbol.  |
`-----------------------------------------------*/

/*ARGSUSED*/
#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
static void
yydestruct (const char *yymsg, int yytype, YYSTYPE *yyvaluep)
#else
static void
yydestruct (yymsg, yytype, yyvaluep)
    const char *yymsg;
    int yytype;
    YYSTYPE *yyvaluep;
#endif
{
  YYUSE (yyvaluep);

  if (!yymsg)
    yymsg = "Deleting";
  YY_SYMBOL_PRINT (yymsg, yytype, yyvaluep, yylocationp);

  switch (yytype)
    {

      default:
        break;
    }
}




/* The lookahead symbol.  */
int yychar;


#ifndef YY_IGNORE_MAYBE_UNINITIALIZED_BEGIN
# define YY_IGNORE_MAYBE_UNINITIALIZED_BEGIN
# define YY_IGNORE_MAYBE_UNINITIALIZED_END
#endif
#ifndef YY_INITIAL_VALUE
# define YY_INITIAL_VALUE(Value) /* Nothing. */
#endif

/* The semantic value of the lookahead symbol.  */
YYSTYPE yylval YY_INITIAL_VALUE(yyval_default);

/* Number of syntax errors so far.  */
int yynerrs;


/*----------.
| yyparse.  |
`----------*/

#ifdef YYPARSE_PARAM
#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
int
yyparse (void *YYPARSE_PARAM)
#else
int
yyparse (YYPARSE_PARAM)
    void *YYPARSE_PARAM;
#endif
#else /* ! YYPARSE_PARAM */
#if (defined __STDC__ || defined __C99__FUNC__ \
     || defined __cplusplus || defined _MSC_VER)
int
yyparse (void)
#else
int
yyparse ()

#endif
#endif
{
    int yystate;
    /* Number of tokens to shift before error messages enabled.  */
    int yyerrstatus;

    /* The stacks and their tools:
       `yyss': related to states.
       `yyvs': related to semantic values.

       Refer to the stacks through separate pointers, to allow yyoverflow
       to reallocate them elsewhere.  */

    /* The state stack.  */
    yytype_int16 yyssa[YYINITDEPTH];
    yytype_int16 *yyss;
    yytype_int16 *yyssp;

    /* The semantic value stack.  */
    YYSTYPE yyvsa[YYINITDEPTH];
    YYSTYPE *yyvs;
    YYSTYPE *yyvsp;

    YYSIZE_T yystacksize;

  int yyn;
  int yyresult;
  /* Lookahead token as an internal (translated) token number.  */
  int yytoken = 0;
  /* The variables used to return semantic value and location from the
     action routines.  */
  YYSTYPE yyval;

#if YYERROR_VERBOSE
  /* Buffer for error messages, and its allocated size.  */
  char yymsgbuf[128];
  char *yymsg = yymsgbuf;
  YYSIZE_T yymsg_alloc = sizeof yymsgbuf;
#endif

#define YYPOPSTACK(N)   (yyvsp -= (N), yyssp -= (N))

  /* The number of symbols on the RHS of the reduced rule.
     Keep to zero when no symbol should be popped.  */
  int yylen = 0;

  yyssp = yyss = yyssa;
  yyvsp = yyvs = yyvsa;
  yystacksize = YYINITDEPTH;

  YYDPRINTF ((stderr, "Starting parse\n"));

  yystate = 0;
  yyerrstatus = 0;
  yynerrs = 0;
  yychar = YYEMPTY; /* Cause a token to be read.  */
  goto yysetstate;

/*------------------------------------------------------------.
| yynewstate -- Push a new state, which is found in yystate.  |
`------------------------------------------------------------*/
 yynewstate:
  /* In all cases, when you get here, the value and location stacks
     have just been pushed.  So pushing a state here evens the stacks.  */
  yyssp++;

 yysetstate:
  *yyssp = yystate;

  if (yyss + yystacksize - 1 <= yyssp)
    {
      /* Get the current used size of the three stacks, in elements.  */
      YYSIZE_T yysize = yyssp - yyss + 1;

#ifdef yyoverflow
      {
	/* Give user a chance to reallocate the stack.  Use copies of
	   these so that the &'s don't force the real ones into
	   memory.  */
	YYSTYPE *yyvs1 = yyvs;
	yytype_int16 *yyss1 = yyss;

	/* Each stack pointer address is followed by the size of the
	   data in use in that stack, in bytes.  This used to be a
	   conditional around just the two extra args, but that might
	   be undefined if yyoverflow is a macro.  */
	yyoverflow (YY_("memory exhausted"),
		    &yyss1, yysize * sizeof (*yyssp),
		    &yyvs1, yysize * sizeof (*yyvsp),
		    &yystacksize);

	yyss = yyss1;
	yyvs = yyvs1;
      }
#else /* no yyoverflow */
# ifndef YYSTACK_RELOCATE
      goto yyexhaustedlab;
# else
      /* Extend the stack our own way.  */
      if (YYMAXDEPTH <= yystacksize)
	goto yyexhaustedlab;
      yystacksize *= 2;
      if (YYMAXDEPTH < yystacksize)
	yystacksize = YYMAXDEPTH;

      {
	yytype_int16 *yyss1 = yyss;
	union yyalloc *yyptr =
	  (union yyalloc *) YYSTACK_ALLOC (YYSTACK_BYTES (yystacksize));
	if (! yyptr)
	  goto yyexhaustedlab;
	YYSTACK_RELOCATE (yyss_alloc, yyss);
	YYSTACK_RELOCATE (yyvs_alloc, yyvs);
#  undef YYSTACK_RELOCATE
	if (yyss1 != yyssa)
	  YYSTACK_FREE (yyss1);
      }
# endif
#endif /* no yyoverflow */

      yyssp = yyss + yysize - 1;
      yyvsp = yyvs + yysize - 1;

      YYDPRINTF ((stderr, "Stack size increased to %lu\n",
		  (unsigned long int) yystacksize));

      if (yyss + yystacksize - 1 <= yyssp)
	YYABORT;
    }

  YYDPRINTF ((stderr, "Entering state %d\n", yystate));

  if (yystate == YYFINAL)
    YYACCEPT;

  goto yybackup;

/*-----------.
| yybackup.  |
`-----------*/
yybackup:

  /* Do appropriate processing given the current state.  Read a
     lookahead token if we need one and don't already have one.  */

  /* First try to decide what to do without reference to lookahead token.  */
  yyn = yypact[yystate];
  if (yypact_value_is_default (yyn))
    goto yydefault;

  /* Not known => get a lookahead token if don't already have one.  */

  /* YYCHAR is either YYEMPTY or YYEOF or a valid lookahead symbol.  */
  if (yychar == YYEMPTY)
    {
      YYDPRINTF ((stderr, "Reading a token: "));
      yychar = YYLEX;
    }

  if (yychar <= YYEOF)
    {
      yychar = yytoken = YYEOF;
      YYDPRINTF ((stderr, "Now at end of input.\n"));
    }
  else
    {
      yytoken = YYTRANSLATE (yychar);
      YY_SYMBOL_PRINT ("Next token is", yytoken, &yylval, &yylloc);
    }

  /* If the proper action on seeing token YYTOKEN is to reduce or to
     detect an error, take that action.  */
  yyn += yytoken;
  if (yyn < 0 || YYLAST < yyn || yycheck[yyn] != yytoken)
    goto yydefault;
  yyn = yytable[yyn];
  if (yyn <= 0)
    {
      if (yytable_value_is_error (yyn))
        goto yyerrlab;
      yyn = -yyn;
      goto yyreduce;
    }

  /* Count tokens shifted since error; after three, turn off error
     status.  */
  if (yyerrstatus)
    yyerrstatus--;

  /* Shift the lookahead token.  */
  YY_SYMBOL_PRINT ("Shifting", yytoken, &yylval, &yylloc);

  /* Discard the shifted token.  */
  yychar = YYEMPTY;

  yystate = yyn;
  YY_IGNORE_MAYBE_UNINITIALIZED_BEGIN
  *++yyvsp = yylval;
  YY_IGNORE_MAYBE_UNINITIALIZED_END

  goto yynewstate;


/*-----------------------------------------------------------.
| yydefault -- do the default action for the current state.  |
`-----------------------------------------------------------*/
yydefault:
  yyn = yydefact[yystate];
  if (yyn == 0)
    goto yyerrlab;
  goto yyreduce;


/*-----------------------------.
| yyreduce -- Do a reduction.  |
`-----------------------------*/
yyreduce:
  /* yyn is the number of a rule to reduce with.  */
  yylen = yyr2[yyn];

  /* If YYLEN is nonzero, implement the default value of the action:
     `$$ = $1'.

     Otherwise, the following line sets YYVAL to garbage.
     This behavior is undocumented and Bison
     users should not rely upon it.  Assigning to YYVAL
     unconditionally makes the parser a bit smaller, and it avoids a
     GCC warning that YYVAL may be used uninitialized.  */
  yyval = yyvsp[1-yylen];


  YY_REDUCE_PRINT (yyn);
  switch (yyn)
    {
        case 2:
/* Line 1792 of yacc.c  */
#line 109 "iris.y"
    {
		IrisCompiler::CurrentCompiler()->AddStatement((yyvsp[(1) - (1)].m_pStatement));
	}
    break;

  case 3:
/* Line 1792 of yacc.c  */
#line 112 "iris.y"
    {
		IrisCompiler::CurrentCompiler()->AddStatement((yyvsp[(2) - (2)].m_pStatement));
	}
    break;

  case 4:
/* Line 1792 of yacc.c  */
#line 118 "iris.y"
    {
		IrisStatement* pStatement = new IrisNormalStatement((yyvsp[(2) - (2)].m_pExpression));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 17:
/* Line 1792 of yacc.c  */
#line 139 "iris.y"
    {
		IrisStatement* pStatement = new IrisClassStatement((yyvsp[(2) - (6)].m_pIdentifier), (yyvsp[(3) - (6)].m_pExpression), (yyvsp[(4) - (6)].m_pExpressionList), (yyvsp[(5) - (6)].m_pExpressionList), (yyvsp[(6) - (6)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 18:
/* Line 1792 of yacc.c  */
#line 147 "iris.y"
    {
		(yyval.m_pBlock) = nullptr;
	}
    break;

  case 19:
/* Line 1792 of yacc.c  */
#line 150 "iris.y"
    {
		(yyval.m_pBlock) = new IrisBlock((yyvsp[(2) - (3)].m_pStatementList));
	}
    break;

  case 20:
/* Line 1792 of yacc.c  */
#line 156 "iris.y"
    {
		IrisList<IrisStatement*>* pList = new IrisList<IrisStatement*>();
		pList->Add((yyvsp[(1) - (1)].m_pStatement));
		(yyval.m_pStatementList) = pList;
	}
    break;

  case 21:
/* Line 1792 of yacc.c  */
#line 161 "iris.y"
    {
		(yyvsp[(1) - (2)].m_pStatementList)->Add((yyvsp[(2) - (2)].m_pStatement));
		(yyval.m_pStatementList) = (yyvsp[(1) - (2)].m_pStatementList);
	}
    break;

  case 22:
/* Line 1792 of yacc.c  */
#line 168 "iris.y"
    {
		IrisStatement* pStatement = new IrisNormalStatement((yyvsp[(2) - (2)].m_pExpression));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 37:
/* Line 1792 of yacc.c  */
#line 190 "iris.y"
    {
		(yyval.m_pExpression) = nullptr;
	}
    break;

  case 38:
/* Line 1792 of yacc.c  */
#line 193 "iris.y"
    {
		(yyval.m_pExpression) = (yyvsp[(2) - (2)].m_pExpression);
	}
    break;

  case 39:
/* Line 1792 of yacc.c  */
#line 196 "iris.y"
    {
		(yyval.m_pExpression) = new IrisFieldExpression(new IrisFieldIdentifier(nullptr, (yyvsp[(2) - (2)].m_pIdentifier), false));
	}
    break;

  case 40:
/* Line 1792 of yacc.c  */
#line 202 "iris.y"
    {
		(yyval.m_pExpressionList) = nullptr;
	}
    break;

  case 41:
/* Line 1792 of yacc.c  */
#line 205 "iris.y"
    {
		(yyval.m_pExpressionList) = (yyvsp[(2) - (2)].m_pExpressionList);
	}
    break;

  case 42:
/* Line 1792 of yacc.c  */
#line 211 "iris.y"
    {
		(yyval.m_pExpressionList) = nullptr;
	}
    break;

  case 43:
/* Line 1792 of yacc.c  */
#line 214 "iris.y"
    {
		(yyval.m_pExpressionList) = (yyvsp[(2) - (2)].m_pExpressionList);
	}
    break;

  case 44:
/* Line 1792 of yacc.c  */
#line 220 "iris.y"
    {
		IrisList<IrisExpression*>* pList = new IrisList<IrisExpression*>();
		pList->Add(new IrisFieldExpression(new IrisFieldIdentifier(nullptr, (yyvsp[(1) - (1)].m_pIdentifier), false)));
		(yyval.m_pExpressionList) = pList;		
	}
    break;

  case 45:
/* Line 1792 of yacc.c  */
#line 225 "iris.y"
    {
		IrisList<IrisExpression*>* pList = new IrisList<IrisExpression*>();
		pList->Add((yyvsp[(1) - (1)].m_pExpression));
		(yyval.m_pExpressionList) = pList;
	}
    break;

  case 46:
/* Line 1792 of yacc.c  */
#line 230 "iris.y"
    {
		(yyvsp[(1) - (3)].m_pExpressionList)->Add(new IrisFieldExpression(new IrisFieldIdentifier(nullptr, (yyvsp[(3) - (3)].m_pIdentifier), false)));
		(yyval.m_pExpressionList) = (yyvsp[(1) - (3)].m_pExpressionList);
	}
    break;

  case 47:
/* Line 1792 of yacc.c  */
#line 234 "iris.y"
    {
		(yyvsp[(1) - (3)].m_pExpressionList)->Add((yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpressionList) = (yyvsp[(1) - (3)].m_pExpressionList);		
	}
    break;

  case 48:
/* Line 1792 of yacc.c  */
#line 242 "iris.y"
    {
		IrisStatement* pStatement = new IrisGetterStatement((yyvsp[(4) - (5)].m_pIdentifier), nullptr);
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 49:
/* Line 1792 of yacc.c  */
#line 247 "iris.y"
    {
		IrisStatement* pStatement = new IrisGetterStatement((yyvsp[(3) - (7)].m_pIdentifier), (yyvsp[(7) - (7)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 50:
/* Line 1792 of yacc.c  */
#line 256 "iris.y"
    {
		IrisStatement* pStatement = new IrisSetterStatement((yyvsp[(4) - (5)].m_pIdentifier), nullptr, nullptr);
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 51:
/* Line 1792 of yacc.c  */
#line 261 "iris.y"
    {
		IrisStatement* pStatement = new IrisSetterStatement((yyvsp[(3) - (8)].m_pIdentifier), (yyvsp[(6) - (8)].m_pIdentifier), (yyvsp[(8) - (8)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 52:
/* Line 1792 of yacc.c  */
#line 270 "iris.y"
    {
		IrisStatement* pStatement = new IrisGSetterStatement((yyvsp[(4) - (5)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
 		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 53:
/* Line 1792 of yacc.c  */
#line 279 "iris.y"
    {
		IrisStatement* pStatement = new IrisAuthorityStatement(IrisAuthorityEnvironment::Class, IrisAuthorityTarget::InstanceMethod, IrisAuthorityType::EveryOne, (yyvsp[(4) - (5)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 54:
/* Line 1792 of yacc.c  */
#line 284 "iris.y"
    {
		IrisStatement* pStatement = new IrisAuthorityStatement(IrisAuthorityEnvironment::Class, IrisAuthorityTarget::ClassMethod, IrisAuthorityType::EveryOne, (yyvsp[(6) - (7)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 55:
/* Line 1792 of yacc.c  */
#line 292 "iris.y"
    {
		IrisStatement* pStatement = new IrisAuthorityStatement(IrisAuthorityEnvironment::Class, IrisAuthorityTarget::InstanceMethod, IrisAuthorityType::Relative, (yyvsp[(4) - (5)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 56:
/* Line 1792 of yacc.c  */
#line 297 "iris.y"
    {
		IrisStatement* pStatement = new IrisAuthorityStatement(IrisAuthorityEnvironment::Class, IrisAuthorityTarget::ClassMethod, IrisAuthorityType::Relative, (yyvsp[(6) - (7)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 57:
/* Line 1792 of yacc.c  */
#line 305 "iris.y"
    {
		IrisStatement* pStatement = new IrisAuthorityStatement(IrisAuthorityEnvironment::Class, IrisAuthorityTarget::InstanceMethod, IrisAuthorityType::Personal, (yyvsp[(4) - (5)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 58:
/* Line 1792 of yacc.c  */
#line 310 "iris.y"
    {
		IrisStatement* pStatement = new IrisAuthorityStatement(IrisAuthorityEnvironment::Class, IrisAuthorityTarget::ClassMethod, IrisAuthorityType::Personal, (yyvsp[(6) - (7)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 59:
/* Line 1792 of yacc.c  */
#line 318 "iris.y"
    {
		IrisStatement* pStatement = new IrisInterfaceStatement((yyvsp[(2) - (4)].m_pIdentifier), (yyvsp[(3) - (4)].m_pExpressionList), (yyvsp[(4) - (4)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 60:
/* Line 1792 of yacc.c  */
#line 326 "iris.y"
    {
		(yyval.m_pBlock) = nullptr;
	}
    break;

  case 61:
/* Line 1792 of yacc.c  */
#line 329 "iris.y"
    {
		(yyval.m_pBlock) = new IrisBlock((yyvsp[(2) - (3)].m_pStatementList));
	}
    break;

  case 62:
/* Line 1792 of yacc.c  */
#line 335 "iris.y"
    {
		IrisList<IrisStatement*>* pList = new IrisList<IrisStatement*>();
		pList->Add((yyvsp[(1) - (1)].m_pStatement));
		(yyval.m_pStatementList) = pList;
	}
    break;

  case 63:
/* Line 1792 of yacc.c  */
#line 340 "iris.y"
    {
		(yyvsp[(1) - (2)].m_pStatementList)->Add((yyvsp[(2) - (2)].m_pStatement));
		(yyval.m_pStatementList) = (yyvsp[(1) - (2)].m_pStatementList);
	}
    break;

  case 64:
/* Line 1792 of yacc.c  */
#line 347 "iris.y"
    {
		IrisStatement* pStatement = new IrisNormalStatement((yyvsp[(2) - (2)].m_pExpression));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 71:
/* Line 1792 of yacc.c  */
#line 362 "iris.y"
    {
		IrisStatement* pStatement = new IrisModuleStatement((yyvsp[(2) - (4)].m_pIdentifier), (yyvsp[(3) - (4)].m_pExpressionList), nullptr, (yyvsp[(4) - (4)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());	
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 72:
/* Line 1792 of yacc.c  */
#line 370 "iris.y"
    {
		(yyval.m_pBlock) = nullptr;
	}
    break;

  case 73:
/* Line 1792 of yacc.c  */
#line 373 "iris.y"
    {
		(yyval.m_pBlock) = new IrisBlock((yyvsp[(2) - (3)].m_pStatementList));
	}
    break;

  case 74:
/* Line 1792 of yacc.c  */
#line 379 "iris.y"
    {
		IrisList<IrisStatement*>* pList = new IrisList<IrisStatement*>();
		pList->Add((yyvsp[(1) - (1)].m_pStatement));
		(yyval.m_pStatementList) = pList;
	}
    break;

  case 75:
/* Line 1792 of yacc.c  */
#line 384 "iris.y"
    {
		(yyvsp[(1) - (2)].m_pStatementList)->Add((yyvsp[(2) - (2)].m_pStatement));
		(yyval.m_pStatementList) = (yyvsp[(1) - (2)].m_pStatementList);
	}
    break;

  case 76:
/* Line 1792 of yacc.c  */
#line 391 "iris.y"
    {
		IrisStatement* pStatement = new IrisNormalStatement((yyvsp[(2) - (2)].m_pExpression));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 89:
/* Line 1792 of yacc.c  */
#line 412 "iris.y"
    {
		IrisStatement* pStatement = new IrisAuthorityStatement(IrisAuthorityEnvironment::Class, IrisAuthorityTarget::InstanceMethod, IrisAuthorityType::EveryOne, (yyvsp[(4) - (5)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 90:
/* Line 1792 of yacc.c  */
#line 417 "iris.y"
    {
		IrisStatement* pStatement = new IrisAuthorityStatement(IrisAuthorityEnvironment::Class, IrisAuthorityTarget::ClassMethod, IrisAuthorityType::EveryOne, (yyvsp[(6) - (7)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 91:
/* Line 1792 of yacc.c  */
#line 425 "iris.y"
    {
		IrisStatement* pStatement = new IrisAuthorityStatement(IrisAuthorityEnvironment::Class, IrisAuthorityTarget::InstanceMethod, IrisAuthorityType::Relative, (yyvsp[(4) - (5)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 92:
/* Line 1792 of yacc.c  */
#line 430 "iris.y"
    {
		IrisStatement* pStatement = new IrisAuthorityStatement(IrisAuthorityEnvironment::Class, IrisAuthorityTarget::ClassMethod, IrisAuthorityType::Relative, (yyvsp[(6) - (7)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 93:
/* Line 1792 of yacc.c  */
#line 438 "iris.y"
    {
		IrisStatement* pStatement = new IrisAuthorityStatement(IrisAuthorityEnvironment::Class, IrisAuthorityTarget::InstanceMethod, IrisAuthorityType::Personal, (yyvsp[(4) - (5)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 94:
/* Line 1792 of yacc.c  */
#line 443 "iris.y"
    {
		IrisStatement* pStatement = new IrisAuthorityStatement(IrisAuthorityEnvironment::Class, IrisAuthorityTarget::ClassMethod, IrisAuthorityType::Personal, (yyvsp[(6) - (7)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 95:
/* Line 1792 of yacc.c  */
#line 451 "iris.y"
    {
		IrisStatement* pStatement = new IrisFunctionStatement((yyvsp[(1) - (2)].m_pFunctionHeader), nullptr, nullptr, (yyvsp[(2) - (2)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 96:
/* Line 1792 of yacc.c  */
#line 456 "iris.y"
    {
		IrisStatement* pStatement = new IrisFunctionStatement((yyvsp[(1) - (6)].m_pFunctionHeader), (yyvsp[(4) - (6)].m_pBlock), (yyvsp[(6) - (6)].m_pBlock), (yyvsp[(2) - (6)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 97:
/* Line 1792 of yacc.c  */
#line 464 "iris.y"
    {
		(yyval.m_pBlock) = nullptr;
	}
    break;

  case 98:
/* Line 1792 of yacc.c  */
#line 467 "iris.y"
    {
		(yyval.m_pBlock) = new IrisBlock((yyvsp[(2) - (3)].m_pStatementList));
	}
    break;

  case 99:
/* Line 1792 of yacc.c  */
#line 473 "iris.y"
    {
		IrisList<IrisStatement*>* pList = new IrisList<IrisStatement*>();
		pList->Add((yyvsp[(1) - (1)].m_pStatement));
		(yyval.m_pStatementList) = pList;
	}
    break;

  case 100:
/* Line 1792 of yacc.c  */
#line 478 "iris.y"
    {
		(yyvsp[(1) - (2)].m_pStatementList)->Add((yyvsp[(2) - (2)].m_pStatement));
		(yyval.m_pStatementList) = (yyvsp[(1) - (2)].m_pStatementList);
	}
    break;

  case 101:
/* Line 1792 of yacc.c  */
#line 485 "iris.y"
    {
		IrisStatement* pStatement = new IrisNormalStatement((yyvsp[(2) - (2)].m_pExpression));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 111:
/* Line 1792 of yacc.c  */
#line 502 "iris.y"
    {
		(yyval.m_pBlock) = nullptr;
	}
    break;

  case 112:
/* Line 1792 of yacc.c  */
#line 505 "iris.y"
    {
		(yyval.m_pBlock) = new IrisBlock((yyvsp[(2) - (3)].m_pStatementList));
	}
    break;

  case 113:
/* Line 1792 of yacc.c  */
#line 511 "iris.y"
    {
		IrisList<IrisStatement*>* pList = new IrisList<IrisStatement*>();
		pList->Add((yyvsp[(1) - (1)].m_pStatement));
		(yyval.m_pStatementList) = pList;
	}
    break;

  case 114:
/* Line 1792 of yacc.c  */
#line 516 "iris.y"
    {
		(yyvsp[(1) - (2)].m_pStatementList)->Add((yyvsp[(2) - (2)].m_pStatement));
		(yyval.m_pStatementList) = (yyvsp[(1) - (2)].m_pStatementList);
	}
    break;

  case 115:
/* Line 1792 of yacc.c  */
#line 523 "iris.y"
    {
		IrisStatement* pStatement = new IrisNormalStatement((yyvsp[(2) - (2)].m_pExpression));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 124:
/* Line 1792 of yacc.c  */
#line 539 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(2) - (4)].m_pIdentifier), nullptr, nullptr, false);
	}
    break;

  case 125:
/* Line 1792 of yacc.c  */
#line 542 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(2) - (5)].m_pIdentifier), (yyvsp[(4) - (5)].m_pIdentifierList), nullptr, false);
	}
    break;

  case 126:
/* Line 1792 of yacc.c  */
#line 545 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(2) - (8)].m_pIdentifier), (yyvsp[(4) - (8)].m_pIdentifierList), (yyvsp[(7) - (8)].m_pIdentifier), false);
	}
    break;

  case 127:
/* Line 1792 of yacc.c  */
#line 548 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(2) - (6)].m_pIdentifier), nullptr, (yyvsp[(5) - (6)].m_pIdentifier), false);
	}
    break;

  case 128:
/* Line 1792 of yacc.c  */
#line 551 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(4) - (6)].m_pIdentifier), nullptr, nullptr, true);
	}
    break;

  case 129:
/* Line 1792 of yacc.c  */
#line 554 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(4) - (7)].m_pIdentifier), (yyvsp[(6) - (7)].m_pIdentifierList), nullptr, true);
	}
    break;

  case 130:
/* Line 1792 of yacc.c  */
#line 557 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(4) - (10)].m_pIdentifier), (yyvsp[(6) - (10)].m_pIdentifierList), (yyvsp[(9) - (10)].m_pIdentifier), true);
	}
    break;

  case 131:
/* Line 1792 of yacc.c  */
#line 560 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(4) - (8)].m_pIdentifier), nullptr, (yyvsp[(7) - (8)].m_pIdentifier), true);
	}
    break;

  case 132:
/* Line 1792 of yacc.c  */
#line 566 "iris.y"
    {
		IrisStatement* pStatement = new IrisFunctionStatement((yyvsp[(1) - (2)].m_pFunctionHeader), nullptr, nullptr, (yyvsp[(2) - (2)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 133:
/* Line 1792 of yacc.c  */
#line 574 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(2) - (4)].m_pIdentifier), nullptr, nullptr, false);
	}
    break;

  case 134:
/* Line 1792 of yacc.c  */
#line 577 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(2) - (5)].m_pIdentifier), (yyvsp[(4) - (5)].m_pIdentifierList), nullptr, false);
	}
    break;

  case 135:
/* Line 1792 of yacc.c  */
#line 588 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "[]");
	}
    break;

  case 136:
/* Line 1792 of yacc.c  */
#line 591 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "[]=");
	}
    break;

  case 137:
/* Line 1792 of yacc.c  */
#line 594 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "+");
	}
    break;

  case 138:
/* Line 1792 of yacc.c  */
#line 597 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "||");
	}
    break;

  case 139:
/* Line 1792 of yacc.c  */
#line 600 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "!");
	}
    break;

  case 140:
/* Line 1792 of yacc.c  */
#line 603 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "==");
	}
    break;

  case 141:
/* Line 1792 of yacc.c  */
#line 606 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "!=");
	}
    break;

  case 142:
/* Line 1792 of yacc.c  */
#line 609 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, ">");
	}
    break;

  case 143:
/* Line 1792 of yacc.c  */
#line 612 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, ">=");
	}
    break;

  case 144:
/* Line 1792 of yacc.c  */
#line 615 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "<");
	}
    break;

  case 145:
/* Line 1792 of yacc.c  */
#line 618 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "<=");
	}
    break;

  case 146:
/* Line 1792 of yacc.c  */
#line 621 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "+");
	}
    break;

  case 147:
/* Line 1792 of yacc.c  */
#line 624 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "-");
	}
    break;

  case 148:
/* Line 1792 of yacc.c  */
#line 627 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "*");
	}
    break;

  case 149:
/* Line 1792 of yacc.c  */
#line 630 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "/");
	}
    break;

  case 150:
/* Line 1792 of yacc.c  */
#line 633 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "%");
	}
    break;

  case 151:
/* Line 1792 of yacc.c  */
#line 636 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "**");
	}
    break;

  case 152:
/* Line 1792 of yacc.c  */
#line 639 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "&");
	}
    break;

  case 153:
/* Line 1792 of yacc.c  */
#line 642 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "|");
	}
    break;

  case 154:
/* Line 1792 of yacc.c  */
#line 645 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "^");
	}
    break;

  case 155:
/* Line 1792 of yacc.c  */
#line 648 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "!");
	}
    break;

  case 156:
/* Line 1792 of yacc.c  */
#line 651 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, ">>");
	}
    break;

  case 157:
/* Line 1792 of yacc.c  */
#line 654 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "<<");
	}
    break;

  case 158:
/* Line 1792 of yacc.c  */
#line 657 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, ">>>");
	}
    break;

  case 159:
/* Line 1792 of yacc.c  */
#line 660 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "<<<");
	}
    break;

  case 160:
/* Line 1792 of yacc.c  */
#line 667 "iris.y"
    {
		(yyvsp[(1) - (1)].m_pConditionIf)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = (yyvsp[(1) - (1)].m_pConditionIf);
	}
    break;

  case 161:
/* Line 1792 of yacc.c  */
#line 671 "iris.y"
    {
		(yyvsp[(1) - (1)].m_pLoopIf)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = (yyvsp[(1) - (1)].m_pLoopIf);
	}
    break;

  case 162:
/* Line 1792 of yacc.c  */
#line 678 "iris.y"
    {
		IrisConditionIfStatement* pConditionIf = new IrisConditionIfStatement((yyvsp[(3) - (5)].m_pExpression), (yyvsp[(5) - (5)].m_pBlock), nullptr, nullptr);
		(yyval.m_pConditionIf) = pConditionIf;
	}
    break;

  case 163:
/* Line 1792 of yacc.c  */
#line 682 "iris.y"
    {
		IrisConditionIfStatement* pConditionIf = new IrisConditionIfStatement((yyvsp[(3) - (7)].m_pExpression), (yyvsp[(5) - (7)].m_pBlock), nullptr, (yyvsp[(7) - (7)].m_pBlock));
		(yyval.m_pConditionIf) = pConditionIf;
	}
    break;

  case 164:
/* Line 1792 of yacc.c  */
#line 687 "iris.y"
    {
		IrisConditionIfStatement* pConditionIf = new IrisConditionIfStatement((yyvsp[(3) - (6)].m_pExpression), (yyvsp[(5) - (6)].m_pBlock), (yyvsp[(6) - (6)].m_pElseIfList), nullptr);
		(yyval.m_pConditionIf) = pConditionIf;
	}
    break;

  case 165:
/* Line 1792 of yacc.c  */
#line 691 "iris.y"
    {
		IrisConditionIfStatement* pConditionIf = new IrisConditionIfStatement((yyvsp[(3) - (8)].m_pExpression), (yyvsp[(5) - (8)].m_pBlock), (yyvsp[(6) - (8)].m_pElseIfList), (yyvsp[(8) - (8)].m_pBlock));
		(yyval.m_pConditionIf) = pConditionIf;
	}
    break;

  case 166:
/* Line 1792 of yacc.c  */
#line 698 "iris.y"
    {
		IrisList<IrisElseIf*>* pElseIfList = new IrisList<IrisElseIf*>();
		pElseIfList->Add((yyvsp[(1) - (1)].m_pElseIf));
		(yyval.m_pElseIfList) = pElseIfList;
	}
    break;

  case 167:
/* Line 1792 of yacc.c  */
#line 703 "iris.y"
    {
		(yyvsp[(1) - (2)].m_pElseIfList)->Add((yyvsp[(2) - (2)].m_pElseIf));
		(yyval.m_pElseIfList) = (yyvsp[(1) - (2)].m_pElseIfList);
	}
    break;

  case 168:
/* Line 1792 of yacc.c  */
#line 710 "iris.y"
    {
		IrisElseIf* pElseIf = new IrisElseIf((yyvsp[(3) - (5)].m_pExpression), (yyvsp[(5) - (5)].m_pBlock));
		(yyval.m_pElseIf) = pElseIf;
	}
    break;

  case 169:
/* Line 1792 of yacc.c  */
#line 717 "iris.y"
    {
		IrisLoopIfStatement* pLoopIf = new IrisLoopIfStatement((yyvsp[(3) - (8)].m_pExpression), (yyvsp[(5) - (8)].m_pExpression), (yyvsp[(6) - (8)].m_pIdentifier), (yyvsp[(8) - (8)].m_pBlock));
		(yyval.m_pLoopIf) = pLoopIf;
	}
    break;

  case 170:
/* Line 1792 of yacc.c  */
#line 724 "iris.y"
    {
		(yyval.m_pIdentifier) = nullptr;
	}
    break;

  case 171:
/* Line 1792 of yacc.c  */
#line 727 "iris.y"
    {
		(yyval.m_pIdentifier) = (yyvsp[(2) - (2)].m_pIdentifier);
	}
    break;

  case 172:
/* Line 1792 of yacc.c  */
#line 734 "iris.y"
    {
		IrisStatement* pStatement = new IrisSwitchStatement((yyvsp[(3) - (5)].m_pExpression), (yyvsp[(5) - (5)].m_pSwitchBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 173:
/* Line 1792 of yacc.c  */
#line 742 "iris.y"
    {
		IrisSwitchBlock* pSwitchBlock = new IrisSwitchBlock((yyvsp[(2) - (3)].m_pWhenList), nullptr);
		(yyval.m_pSwitchBlock) = pSwitchBlock;
	}
    break;

  case 174:
/* Line 1792 of yacc.c  */
#line 746 "iris.y"
    {
		IrisSwitchBlock* pSwitchBlock = new IrisSwitchBlock((yyvsp[(2) - (5)].m_pWhenList), (yyvsp[(4) - (5)].m_pBlock));
		(yyval.m_pSwitchBlock) = pSwitchBlock;		
	}
    break;

  case 175:
/* Line 1792 of yacc.c  */
#line 753 "iris.y"
    {
		IrisList<IrisWhen*>* pWhenList = new IrisList<IrisWhen*>();
		pWhenList->Add((yyvsp[(1) - (1)].m_pWhen));
		(yyval.m_pWhenList) = pWhenList;
	}
    break;

  case 176:
/* Line 1792 of yacc.c  */
#line 758 "iris.y"
    {
		(yyvsp[(1) - (2)].m_pWhenList)->Add((yyvsp[(2) - (2)].m_pWhen));
		(yyval.m_pWhenList) = (yyvsp[(1) - (2)].m_pWhenList);
	}
    break;

  case 177:
/* Line 1792 of yacc.c  */
#line 765 "iris.y"
    {
		IrisWhen* pWhen = new IrisWhen((yyvsp[(3) - (5)].m_pExpressionList), (yyvsp[(5) - (5)].m_pBlock));
		(yyval.m_pWhen) = pWhen;
	}
    break;

  case 178:
/* Line 1792 of yacc.c  */
#line 773 "iris.y"
    {
		IrisStatement* pStatement = new IrisForStatement((yyvsp[(3) - (7)].m_pIdentifier), nullptr, (yyvsp[(5) - (7)].m_pExpression), (yyvsp[(7) - (7)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 179:
/* Line 1792 of yacc.c  */
#line 778 "iris.y"
    {
		IrisStatement* pStatement = new IrisForStatement((yyvsp[(4) - (11)].m_pIdentifier), (yyvsp[(6) - (11)].m_pIdentifier), (yyvsp[(9) - (11)].m_pExpression), (yyvsp[(11) - (11)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 180:
/* Line 1792 of yacc.c  */
#line 787 "iris.y"
    {
		IrisStatement* pStatement = new IrisOrderStatement((yyvsp[(2) - (8)].m_pBlock), (yyvsp[(5) - (8)].m_pIdentifier), (yyvsp[(7) - (8)].m_pBlock), (yyvsp[(8) - (8)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 181:
/* Line 1792 of yacc.c  */
#line 795 "iris.y"
    {
		(yyval.m_pBlock) = nullptr;
	}
    break;

  case 182:
/* Line 1792 of yacc.c  */
#line 798 "iris.y"
    {
		(yyval.m_pBlock) = new IrisBlock((yyvsp[(2) - (3)].m_pStatementList));
	}
    break;

  case 183:
/* Line 1792 of yacc.c  */
#line 804 "iris.y"
    {
		IrisList<IrisStatement*>* pList = new IrisList<IrisStatement*>();
		pList->Add((yyvsp[(1) - (1)].m_pStatement));
		(yyval.m_pStatementList) = pList;
	}
    break;

  case 184:
/* Line 1792 of yacc.c  */
#line 809 "iris.y"
    {
		(yyvsp[(1) - (2)].m_pStatementList)->Add((yyvsp[(2) - (2)].m_pStatement));
		(yyval.m_pStatementList) = (yyvsp[(1) - (2)].m_pStatementList);
	}
    break;

  case 185:
/* Line 1792 of yacc.c  */
#line 816 "iris.y"
    {
		IrisStatement* pStatement = new IrisNormalStatement((yyvsp[(2) - (2)].m_pExpression));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 193:
/* Line 1792 of yacc.c  */
#line 832 "iris.y"
    {
		(yyval.m_pBlock) = nullptr;
	}
    break;

  case 194:
/* Line 1792 of yacc.c  */
#line 835 "iris.y"
    {
		(yyval.m_pBlock) = new IrisBlock((yyvsp[(3) - (4)].m_pStatementList));;
	}
    break;

  case 195:
/* Line 1792 of yacc.c  */
#line 843 "iris.y"
    {
		IrisStatement* pStatement = new IrisReturnStatement(nullptr);
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 196:
/* Line 1792 of yacc.c  */
#line 848 "iris.y"
    {
		IrisStatement* pStatement = new IrisReturnStatement((yyvsp[(3) - (3)].m_pExpression));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 197:
/* Line 1792 of yacc.c  */
#line 857 "iris.y"
    {
		IrisStatement* pStatement = new IrisBreakStatement(nullptr);
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 198:
/* Line 1792 of yacc.c  */
#line 865 "iris.y"
    {
		IrisStatement* pStatement = new IrisContinueStatement(nullptr);
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 199:
/* Line 1792 of yacc.c  */
#line 874 "iris.y"
    {
		IrisStatement* pStatement = new IrisInterfaceFunctionStatement((yyvsp[(3) - (5)].m_pIdentifier), nullptr, nullptr);
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 200:
/* Line 1792 of yacc.c  */
#line 879 "iris.y"
    {
		IrisStatement* pStatement = new IrisInterfaceFunctionStatement((yyvsp[(3) - (6)].m_pIdentifier), (yyvsp[(5) - (6)].m_pIdentifierList), nullptr);
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 201:
/* Line 1792 of yacc.c  */
#line 884 "iris.y"
    {
		IrisStatement* pStatement = new IrisInterfaceFunctionStatement((yyvsp[(3) - (9)].m_pIdentifier), (yyvsp[(5) - (9)].m_pIdentifierList), (yyvsp[(8) - (9)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 202:
/* Line 1792 of yacc.c  */
#line 889 "iris.y"
    {
		IrisStatement* pStatement = new IrisInterfaceFunctionStatement((yyvsp[(3) - (7)].m_pIdentifier), nullptr, (yyvsp[(6) - (7)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 203:
/* Line 1792 of yacc.c  */
#line 898 "iris.y"
    {
		IrisStatement* pStatement = new IrisGroanStatement((yyvsp[(4) - (5)].m_pExpression));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 204:
/* Line 1792 of yacc.c  */
#line 907 "iris.y"
    {
		IrisStatement* pStatement = new IrisBlockStatement();
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 205:
/* Line 1792 of yacc.c  */
#line 916 "iris.y"
    {
		IrisStatement* pStatement = new IrisCastStatement(nullptr);
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 206:
/* Line 1792 of yacc.c  */
#line 921 "iris.y"
    {
		IrisStatement* pStatement = new IrisCastStatement((yyvsp[(4) - (5)].m_pExpressionList));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 207:
/* Line 1792 of yacc.c  */
#line 930 "iris.y"
    {
		IrisStatement* pStatement = new IrisSuperStatement(nullptr);
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 208:
/* Line 1792 of yacc.c  */
#line 935 "iris.y"
    {
		IrisStatement* pStatement = new IrisSuperStatement((yyvsp[(4) - (5)].m_pExpressionList));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;	
	}
    break;

  case 209:
/* Line 1792 of yacc.c  */
#line 943 "iris.y"
    {
		IrisBlock* pBlock = new IrisBlock(nullptr);
		(yyval.m_pBlock) = pBlock;
	}
    break;

  case 210:
/* Line 1792 of yacc.c  */
#line 947 "iris.y"
    {
		IrisBlock* pBlock = new IrisBlock((yyvsp[(2) - (3)].m_pStatementList));
		(yyval.m_pBlock) = pBlock;		
	}
    break;

  case 211:
/* Line 1792 of yacc.c  */
#line 953 "iris.y"
    {
		IrisList<IrisStatement*>* pStatementList = new IrisList<IrisStatement*>();
		pStatementList->Add((yyvsp[(1) - (1)].m_pStatement));
		(yyval.m_pStatementList) = pStatementList;
	}
    break;

  case 212:
/* Line 1792 of yacc.c  */
#line 958 "iris.y"
    {
		(yyvsp[(1) - (2)].m_pStatementList)->Add((yyvsp[(2) - (2)].m_pStatement));
		(yyval.m_pStatementList) = (yyvsp[(1) - (2)].m_pStatementList);
	}
    break;

  case 214:
/* Line 1792 of yacc.c  */
#line 967 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression((yyvsp[(2) - (3)].m_eExpressionType), (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 215:
/* Line 1792 of yacc.c  */
#line 974 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::Assign;
	}
    break;

  case 216:
/* Line 1792 of yacc.c  */
#line 977 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignAdd;
	}
    break;

  case 217:
/* Line 1792 of yacc.c  */
#line 980 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignSub;
	}
    break;

  case 218:
/* Line 1792 of yacc.c  */
#line 983 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignMul;
	}
    break;

  case 219:
/* Line 1792 of yacc.c  */
#line 986 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignDiv;
	}
    break;

  case 220:
/* Line 1792 of yacc.c  */
#line 989 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignMod;
	}
    break;

  case 221:
/* Line 1792 of yacc.c  */
#line 992 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignBitAnd;
	}
    break;

  case 222:
/* Line 1792 of yacc.c  */
#line 995 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignBitOr;
	}
    break;

  case 223:
/* Line 1792 of yacc.c  */
#line 998 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignBitXor;
	}
    break;

  case 224:
/* Line 1792 of yacc.c  */
#line 1001 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignBitShr;
	}
    break;

  case 225:
/* Line 1792 of yacc.c  */
#line 1004 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignBitShl;
	}
    break;

  case 226:
/* Line 1792 of yacc.c  */
#line 1007 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignBitSar;
	}
    break;

  case 227:
/* Line 1792 of yacc.c  */
#line 1010 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignBitSal;
	}
    break;

  case 229:
/* Line 1792 of yacc.c  */
#line 1017 "iris.y"
    {
		(yyval.m_pExpression) =  new IrisBinaryExpression(IrisBinaryExpressionType::LogicOr, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 231:
/* Line 1792 of yacc.c  */
#line 1025 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::LogicAnd, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 233:
/* Line 1792 of yacc.c  */
#line 1033 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::LogicBitOr, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 235:
/* Line 1792 of yacc.c  */
#line 1041 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::LogicBitXor, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 237:
/* Line 1792 of yacc.c  */
#line 1049 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::LogicBitAnd, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 239:
/* Line 1792 of yacc.c  */
#line 1057 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::Equal, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 240:
/* Line 1792 of yacc.c  */
#line 1061 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::NotEqual, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 242:
/* Line 1792 of yacc.c  */
#line 1069 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::GreatThan, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 243:
/* Line 1792 of yacc.c  */
#line 1073 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::GreatThanOrEqual, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 244:
/* Line 1792 of yacc.c  */
#line 1077 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::LessThan, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 245:
/* Line 1792 of yacc.c  */
#line 1081 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::LessThanOrEqual, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 247:
/* Line 1792 of yacc.c  */
#line 1089 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::BitShr, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 248:
/* Line 1792 of yacc.c  */
#line 1093 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::BitShl, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 249:
/* Line 1792 of yacc.c  */
#line 1097 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::BitSar, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 250:
/* Line 1792 of yacc.c  */
#line 1101 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::BitSal, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 252:
/* Line 1792 of yacc.c  */
#line 1109 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::Add, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 253:
/* Line 1792 of yacc.c  */
#line 1113 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::Sub, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 255:
/* Line 1792 of yacc.c  */
#line 1121 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::Mul, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 256:
/* Line 1792 of yacc.c  */
#line 1125 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::Div, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 257:
/* Line 1792 of yacc.c  */
#line 1129 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::Mod, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 259:
/* Line 1792 of yacc.c  */
#line 1137 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::Power, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 261:
/* Line 1792 of yacc.c  */
#line 1145 "iris.y"
    {
		(yyval.m_pExpression) = new IrisUnaryExpression(IrisUnaryExpressionType::LogicNot, (yyvsp[(2) - (2)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 262:
/* Line 1792 of yacc.c  */
#line 1149 "iris.y"
    {
		(yyval.m_pExpression) = new IrisUnaryExpression(IrisUnaryExpressionType::BitNot, (yyvsp[(2) - (2)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 263:
/* Line 1792 of yacc.c  */
#line 1153 "iris.y"
    {
		(yyval.m_pExpression) = new IrisUnaryExpression(IrisUnaryExpressionType::Minus, (yyvsp[(2) - (2)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 264:
/* Line 1792 of yacc.c  */
#line 1157 "iris.y"
    {
		(yyval.m_pExpression) = new IrisUnaryExpression(IrisUnaryExpressionType::Plus, (yyvsp[(2) - (2)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 266:
/* Line 1792 of yacc.c  */
#line 1165 "iris.y"
    {
		(yyval.m_pExpression) = new IrisIndexExpression((yyvsp[(1) - (4)].m_pExpression), (yyvsp[(3) - (4)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 267:
/* Line 1792 of yacc.c  */
#line 1169 "iris.y"
    {
		(yyval.m_pExpression) = new IrisFunctionCallExpression((yyvsp[(1) - (6)].m_pExpression), (yyvsp[(3) - (6)].m_pIdentifier), nullptr, (yyvsp[(6) - (6)].m_pClosureBlockLiteral));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 268:
/* Line 1792 of yacc.c  */
#line 1173 "iris.y"
    {
		(yyval.m_pExpression) = new IrisFunctionCallExpression((yyvsp[(1) - (7)].m_pExpression), (yyvsp[(3) - (7)].m_pIdentifier), (yyvsp[(5) - (7)].m_pExpressionList), (yyvsp[(7) - (7)].m_pClosureBlockLiteral));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 269:
/* Line 1792 of yacc.c  */
#line 1187 "iris.y"
    {
		(yyval.m_pExpression) = new IrisFunctionCallExpression(nullptr, (yyvsp[(1) - (4)].m_pIdentifier), nullptr, (yyvsp[(4) - (4)].m_pClosureBlockLiteral));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 270:
/* Line 1792 of yacc.c  */
#line 1191 "iris.y"
    {
		(yyval.m_pExpression) = new IrisFunctionCallExpression(nullptr, (yyvsp[(1) - (5)].m_pIdentifier), (yyvsp[(3) - (5)].m_pExpressionList), (yyvsp[(5) - (5)].m_pClosureBlockLiteral));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 271:
/* Line 1792 of yacc.c  */
#line 1195 "iris.y"
    {
		(yyval.m_pExpression) = new IrisMemberExpression((yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pIdentifier));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 272:
/* Line 1792 of yacc.c  */
#line 1208 "iris.y"
    {
		(yyval.m_pClosureBlockLiteral) = nullptr;
	}
    break;

  case 273:
/* Line 1792 of yacc.c  */
#line 1211 "iris.y"
    {
		(yyval.m_pClosureBlockLiteral) = nullptr;
	}
    break;

  case 274:
/* Line 1792 of yacc.c  */
#line 1214 "iris.y"
    {
		(yyval.m_pClosureBlockLiteral) = new IrisClosureBlockLiteral(nullptr, (yyvsp[(2) - (3)].m_pStatementList));
	}
    break;

  case 275:
/* Line 1792 of yacc.c  */
#line 1217 "iris.y"
    {
		(yyval.m_pClosureBlockLiteral) = new IrisClosureBlockLiteral((yyvsp[(5) - (9)].m_pIdentifierList), (yyvsp[(8) - (9)].m_pStatementList));
	}
    break;

  case 276:
/* Line 1792 of yacc.c  */
#line 1223 "iris.y"
    {
		IrisList<IrisStatement*>* pList = new IrisList<IrisStatement*>();
		pList->Add((yyvsp[(1) - (1)].m_pStatement));
		(yyval.m_pStatementList) = pList;
	}
    break;

  case 277:
/* Line 1792 of yacc.c  */
#line 1228 "iris.y"
    {
		(yyvsp[(1) - (2)].m_pStatementList)->Add((yyvsp[(2) - (2)].m_pStatement));
		(yyval.m_pStatementList) = (yyvsp[(1) - (2)].m_pStatementList);
	}
    break;

  case 278:
/* Line 1792 of yacc.c  */
#line 1235 "iris.y"
    {
		IrisStatement* pStatement = new IrisNormalStatement((yyvsp[(2) - (2)].m_pExpression));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());	
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 285:
/* Line 1792 of yacc.c  */
#line 1249 "iris.y"
    {
		(yyval.m_pExpression) = (yyvsp[(2) - (3)].m_pExpression);
	}
    break;

  case 286:
/* Line 1792 of yacc.c  */
#line 1252 "iris.y"
    {
		(yyval.m_pExpression) = new IrisIdentifierExpression((yyvsp[(1) - (1)].m_pIdentifier));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 287:
/* Line 1792 of yacc.c  */
#line 1256 "iris.y"
    {
		(yyval.m_pExpression) = (yyvsp[(1) - (1)].m_pExpression);
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 288:
/* Line 1792 of yacc.c  */
#line 1260 "iris.y"
    {
		(yyval.m_pExpression) = (yyvsp[(1) - (1)].m_pExpression);
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 289:
/* Line 1792 of yacc.c  */
#line 1264 "iris.y"
    {
		(yyval.m_pExpression) = (yyvsp[(1) - (1)].m_pExpression);
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 290:
/* Line 1792 of yacc.c  */
#line 1268 "iris.y"
    {
		(yyval.m_pExpression) = new IrisInstantValueExpression(IrisInstantValueType::True);
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 291:
/* Line 1792 of yacc.c  */
#line 1272 "iris.y"
    {
		(yyval.m_pExpression) = new IrisInstantValueExpression(IrisInstantValueType::False);
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 292:
/* Line 1792 of yacc.c  */
#line 1276 "iris.y"
    {
		(yyval.m_pExpression) = new IrisInstantValueExpression(IrisInstantValueType::Nil);
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 293:
/* Line 1792 of yacc.c  */
#line 1280 "iris.y"
    {
		(yyval.m_pExpression) = new IrisSelfExpression();
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 297:
/* Line 1792 of yacc.c  */
#line 1290 "iris.y"
    {
		(yyval.m_pExpression) = new IrisFieldExpression(new IrisFieldIdentifier((yyvsp[(1) - (2)].m_pIdentifierList), (yyvsp[(2) - (2)].m_pIdentifier), false));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 298:
/* Line 1792 of yacc.c  */
#line 1294 "iris.y"
    {
		(yyval.m_pExpression) = new IrisFieldExpression(new IrisFieldIdentifier((yyvsp[(2) - (3)].m_pIdentifierList), (yyvsp[(3) - (3)].m_pIdentifier), true));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 299:
/* Line 1792 of yacc.c  */
#line 1298 "iris.y"
    {
		(yyval.m_pExpression) = new IrisFieldExpression(new IrisFieldIdentifier(nullptr, (yyvsp[(2) - (2)].m_pIdentifier), true));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 300:
/* Line 1792 of yacc.c  */
#line 1305 "iris.y"
    {
		(yyval.m_pExpression) = new IrisArrayExpression(nullptr);
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 301:
/* Line 1792 of yacc.c  */
#line 1309 "iris.y"
    {
        (yyval.m_pExpression) = new IrisArrayExpression((yyvsp[(2) - (3)].m_pExpressionList));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
    }
    break;

  case 302:
/* Line 1792 of yacc.c  */
#line 1313 "iris.y"
    {
        (yyval.m_pExpression) = new IrisArrayExpression((yyvsp[(2) - (4)].m_pExpressionList));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
    }
    break;

  case 303:
/* Line 1792 of yacc.c  */
#line 1321 "iris.y"
    {
		IrisList<IrisIdentifier*>* pFieldIdentifier = new IrisList<IrisIdentifier*>();
		pFieldIdentifier->Add((yyvsp[(1) - (2)].m_pIdentifier));
		(yyval.m_pIdentifierList) = pFieldIdentifier;
	}
    break;

  case 304:
/* Line 1792 of yacc.c  */
#line 1326 "iris.y"
    {
		(yyvsp[(1) - (3)].m_pIdentifierList)->Add((yyvsp[(2) - (3)].m_pIdentifier));
		(yyval.m_pIdentifierList) = (yyvsp[(1) - (3)].m_pIdentifierList);
	}
    break;

  case 305:
/* Line 1792 of yacc.c  */
#line 1334 "iris.y"
    {
		IrisList<IrisIdentifier*>* pList = new IrisList<IrisIdentifier*>();
		pList->Add((yyvsp[(1) - (1)].m_pIdentifier));
		(yyval.m_pIdentifierList) = pList;
	}
    break;

  case 306:
/* Line 1792 of yacc.c  */
#line 1339 "iris.y"
    {
		(yyvsp[(1) - (3)].m_pIdentifierList)->Add((yyvsp[(3) - (3)].m_pIdentifier));
		(yyval.m_pIdentifierList) = (yyvsp[(1) - (3)].m_pIdentifierList);
	}
    break;

  case 307:
/* Line 1792 of yacc.c  */
#line 1346 "iris.y"
    {
		IrisList<IrisExpression*>* pExpressionList = new IrisList<IrisExpression*>();
		pExpressionList->Add((yyvsp[(1) - (1)].m_pExpression));
		(yyval.m_pExpressionList) = pExpressionList;
	}
    break;

  case 308:
/* Line 1792 of yacc.c  */
#line 1351 "iris.y"
    {
		(yyvsp[(1) - (3)].m_pExpressionList)->Add((yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpressionList) = (yyvsp[(1) - (3)].m_pExpressionList);
	}
    break;

  case 309:
/* Line 1792 of yacc.c  */
#line 1358 "iris.y"
    {
		(yyval.m_pExpression) = new IrisHashExpression(nullptr);
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 310:
/* Line 1792 of yacc.c  */
#line 1362 "iris.y"
    {
        (yyval.m_pExpression) = new IrisHashExpression((yyvsp[(2) - (3)].m_pHashPairList));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
    }
    break;

  case 311:
/* Line 1792 of yacc.c  */
#line 1366 "iris.y"
    {
		(yyvsp[(2) - (6)].m_pHashPairList)->Add(new IrisHashPair((yyvsp[(3) - (6)].m_pExpression), (yyvsp[(5) - (6)].m_pExpression)));
        (yyval.m_pExpression) = new IrisHashExpression((yyvsp[(2) - (6)].m_pHashPairList));
    }
    break;

  case 312:
/* Line 1792 of yacc.c  */
#line 1373 "iris.y"
    {
		IrisList<IrisHashPair*>* pHashPairList = new IrisList<IrisHashPair*>();
		pHashPairList->Add((yyvsp[(1) - (1)].m_pHashPair));
		(yyval.m_pHashPairList) = pHashPairList;
	}
    break;

  case 313:
/* Line 1792 of yacc.c  */
#line 1378 "iris.y"
    {
		(yyvsp[(1) - (2)].m_pHashPairList)->Add((yyvsp[(2) - (2)].m_pHashPair));
		(yyval.m_pHashPairList) = (yyvsp[(1) - (2)].m_pHashPairList);
	}
    break;

  case 314:
/* Line 1792 of yacc.c  */
#line 1384 "iris.y"
    {
		(yyval.m_pHashPair) = new IrisHashPair((yyvsp[(1) - (4)].m_pExpression), (yyvsp[(3) - (4)].m_pExpression));
	}
    break;

  case 315:
/* Line 1792 of yacc.c  */
#line 1389 "iris.y"
    {
		(yyval.m_pExpression) = new IrisRangeExpression(IrisRangeType::LeftOpenAndRightOpen, (yyvsp[(2) - (5)].m_pExpression), (yyvsp[(4) - (5)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 316:
/* Line 1792 of yacc.c  */
#line 1393 "iris.y"
    {
		(yyval.m_pExpression) = new IrisRangeExpression(IrisRangeType::LeftOpenAndRightClosed, (yyvsp[(2) - (5)].m_pExpression), (yyvsp[(4) - (5)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());	
	}
    break;

  case 317:
/* Line 1792 of yacc.c  */
#line 1397 "iris.y"
    {
		(yyval.m_pExpression) = new IrisRangeExpression(IrisRangeType::LeftClosedAndRightClosed, (yyvsp[(2) - (5)].m_pExpression), (yyvsp[(4) - (5)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());		
	}
    break;

  case 318:
/* Line 1792 of yacc.c  */
#line 1401 "iris.y"
    {
		(yyval.m_pExpression) = new IrisRangeExpression(IrisRangeType::LeftClosedAndRightOpen, (yyvsp[(2) - (5)].m_pExpression), (yyvsp[(4) - (5)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());			
	}
    break;


/* Line 1792 of yacc.c  */
#line 4329 "y.tab.c"
      default: break;
    }
  /* User semantic actions sometimes alter yychar, and that requires
     that yytoken be updated with the new translation.  We take the
     approach of translating immediately before every use of yytoken.
     One alternative is translating here after every semantic action,
     but that translation would be missed if the semantic action invokes
     YYABORT, YYACCEPT, or YYERROR immediately after altering yychar or
     if it invokes YYBACKUP.  In the case of YYABORT or YYACCEPT, an
     incorrect destructor might then be invoked immediately.  In the
     case of YYERROR or YYBACKUP, subsequent parser actions might lead
     to an incorrect destructor call or verbose syntax error message
     before the lookahead is translated.  */
  YY_SYMBOL_PRINT ("-> $$ =", yyr1[yyn], &yyval, &yyloc);

  YYPOPSTACK (yylen);
  yylen = 0;
  YY_STACK_PRINT (yyss, yyssp);

  *++yyvsp = yyval;

  /* Now `shift' the result of the reduction.  Determine what state
     that goes to, based on the state we popped back to and the rule
     number reduced by.  */

  yyn = yyr1[yyn];

  yystate = yypgoto[yyn - YYNTOKENS] + *yyssp;
  if (0 <= yystate && yystate <= YYLAST && yycheck[yystate] == *yyssp)
    yystate = yytable[yystate];
  else
    yystate = yydefgoto[yyn - YYNTOKENS];

  goto yynewstate;


/*------------------------------------.
| yyerrlab -- here on detecting error |
`------------------------------------*/
yyerrlab:
  /* Make sure we have latest lookahead translation.  See comments at
     user semantic actions for why this is necessary.  */
  yytoken = yychar == YYEMPTY ? YYEMPTY : YYTRANSLATE (yychar);

  /* If not already recovering from an error, report this error.  */
  if (!yyerrstatus)
    {
      ++yynerrs;
#if ! YYERROR_VERBOSE
      yyerror (YY_("syntax error"));
#else
# define YYSYNTAX_ERROR yysyntax_error (&yymsg_alloc, &yymsg, \
                                        yyssp, yytoken)
      {
        char const *yymsgp = YY_("syntax error");
        int yysyntax_error_status;
        yysyntax_error_status = YYSYNTAX_ERROR;
        if (yysyntax_error_status == 0)
          yymsgp = yymsg;
        else if (yysyntax_error_status == 1)
          {
            if (yymsg != yymsgbuf)
              YYSTACK_FREE (yymsg);
            yymsg = (char *) YYSTACK_ALLOC (yymsg_alloc);
            if (!yymsg)
              {
                yymsg = yymsgbuf;
                yymsg_alloc = sizeof yymsgbuf;
                yysyntax_error_status = 2;
              }
            else
              {
                yysyntax_error_status = YYSYNTAX_ERROR;
                yymsgp = yymsg;
              }
          }
        yyerror (yymsgp);
        if (yysyntax_error_status == 2)
          goto yyexhaustedlab;
      }
# undef YYSYNTAX_ERROR
#endif
    }



  if (yyerrstatus == 3)
    {
      /* If just tried and failed to reuse lookahead token after an
	 error, discard it.  */

      if (yychar <= YYEOF)
	{
	  /* Return failure if at end of input.  */
	  if (yychar == YYEOF)
	    YYABORT;
	}
      else
	{
	  yydestruct ("Error: discarding",
		      yytoken, &yylval);
	  yychar = YYEMPTY;
	}
    }

  /* Else will try to reuse lookahead token after shifting the error
     token.  */
  goto yyerrlab1;


/*---------------------------------------------------.
| yyerrorlab -- error raised explicitly by YYERROR.  |
`---------------------------------------------------*/
yyerrorlab:

  /* Pacify compilers like GCC when the user code never invokes
     YYERROR and the label yyerrorlab therefore never appears in user
     code.  */
  if (/*CONSTCOND*/ 0)
     goto yyerrorlab;

  /* Do not reclaim the symbols of the rule which action triggered
     this YYERROR.  */
  YYPOPSTACK (yylen);
  yylen = 0;
  YY_STACK_PRINT (yyss, yyssp);
  yystate = *yyssp;
  goto yyerrlab1;


/*-------------------------------------------------------------.
| yyerrlab1 -- common code for both syntax error and YYERROR.  |
`-------------------------------------------------------------*/
yyerrlab1:
  yyerrstatus = 3;	/* Each real token shifted decrements this.  */

  for (;;)
    {
      yyn = yypact[yystate];
      if (!yypact_value_is_default (yyn))
	{
	  yyn += YYTERROR;
	  if (0 <= yyn && yyn <= YYLAST && yycheck[yyn] == YYTERROR)
	    {
	      yyn = yytable[yyn];
	      if (0 < yyn)
		break;
	    }
	}

      /* Pop the current state because it cannot handle the error token.  */
      if (yyssp == yyss)
	YYABORT;


      yydestruct ("Error: popping",
		  yystos[yystate], yyvsp);
      YYPOPSTACK (1);
      yystate = *yyssp;
      YY_STACK_PRINT (yyss, yyssp);
    }

  YY_IGNORE_MAYBE_UNINITIALIZED_BEGIN
  *++yyvsp = yylval;
  YY_IGNORE_MAYBE_UNINITIALIZED_END


  /* Shift the error token.  */
  YY_SYMBOL_PRINT ("Shifting", yystos[yyn], yyvsp, yylsp);

  yystate = yyn;
  goto yynewstate;


/*-------------------------------------.
| yyacceptlab -- YYACCEPT comes here.  |
`-------------------------------------*/
yyacceptlab:
  yyresult = 0;
  goto yyreturn;

/*-----------------------------------.
| yyabortlab -- YYABORT comes here.  |
`-----------------------------------*/
yyabortlab:
  yyresult = 1;
  goto yyreturn;

#if !defined yyoverflow || YYERROR_VERBOSE
/*-------------------------------------------------.
| yyexhaustedlab -- memory exhaustion comes here.  |
`-------------------------------------------------*/
yyexhaustedlab:
  yyerror (YY_("memory exhausted"));
  yyresult = 2;
  /* Fall through.  */
#endif

yyreturn:
  if (yychar != YYEMPTY)
    {
      /* Make sure we have latest lookahead translation.  See comments at
         user semantic actions for why this is necessary.  */
      yytoken = YYTRANSLATE (yychar);
      yydestruct ("Cleanup: discarding lookahead",
                  yytoken, &yylval);
    }
  /* Do not reclaim the symbols of the rule which action triggered
     this YYABORT or YYACCEPT.  */
  YYPOPSTACK (yylen);
  YY_STACK_PRINT (yyss, yyssp);
  while (yyssp != yyss)
    {
      yydestruct ("Cleanup: popping",
		  yystos[*yyssp], yyvsp);
      YYPOPSTACK (1);
    }
#ifndef yyoverflow
  if (yyss != yyssa)
    YYSTACK_FREE (yyss);
#endif
#if YYERROR_VERBOSE
  if (yymsg != yymsgbuf)
    YYSTACK_FREE (yymsg);
#endif
  /* Make sure YYID is used.  */
  return YYID (yyresult);
}


/* Line 2055 of yacc.c  */
#line 1405 "iris.y"
