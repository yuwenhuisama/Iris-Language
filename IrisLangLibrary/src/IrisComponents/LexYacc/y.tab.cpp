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
	string strLinenoMessage = ">and happened at line " + ssStream.str() + " where token is " + string(yytext) + ".";

	strMessage += strIrregularMessage + "SyntaxIrregular," + "\n" + strLinenoMessage + "\n";

	IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorDirectly(strMessage);
	return 0;
}

/* Line 371 of yacc.c  */
#line 93 "y.tab.c"

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
#line 26 "iris.y"

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
#line 357 "y.tab.c"
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
#line 385 "y.tab.c"

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
#define YYFINAL  129
/* YYLAST -- Last index in YYTABLE.  */
#define YYLAST   878

/* YYNTOKENS -- Number of terminals.  */
#define YYNTOKENS  100
/* YYNNTS -- Number of nonterminals.  */
#define YYNNTS  71
/* YYNRULES -- Number of rules.  */
#define YYNRULES  231
/* YYNRULES -- Number of states.  */
#define YYNSTATES  451

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
       0,     0,     3,     4,     6,     8,    11,    14,    16,    18,
      20,    22,    24,    26,    28,    30,    32,    34,    36,    38,
      40,    42,    44,    46,    48,    50,    52,    54,    56,    58,
      60,    67,    68,    71,    74,    75,    78,    79,    82,    84,
      86,    90,    94,   100,   108,   114,   123,   129,   135,   143,
     149,   157,   163,   171,   176,   181,   184,   191,   196,   202,
     211,   218,   225,   233,   244,   253,   256,   261,   267,   270,
     274,   276,   278,   280,   282,   284,   286,   288,   290,   292,
     294,   296,   298,   300,   302,   304,   306,   308,   310,   312,
     314,   316,   318,   320,   322,   324,   330,   338,   345,   354,
     356,   359,   365,   374,   375,   378,   384,   388,   394,   396,
     399,   405,   413,   425,   434,   435,   440,   443,   447,   450,
     453,   459,   466,   476,   484,   490,   493,   498,   504,   509,
     515,   518,   522,   524,   527,   529,   533,   535,   537,   539,
     541,   543,   545,   547,   549,   551,   553,   555,   557,   559,
     561,   565,   567,   571,   573,   577,   579,   583,   585,   589,
     591,   595,   599,   601,   605,   609,   613,   617,   619,   623,
     627,   631,   635,   637,   641,   645,   647,   651,   655,   659,
     661,   665,   667,   670,   673,   676,   679,   681,   686,   693,
     701,   706,   712,   716,   717,   720,   724,   734,   736,   740,
     742,   744,   746,   748,   750,   752,   754,   756,   758,   760,
     762,   765,   769,   772,   775,   779,   784,   787,   791,   793,
     797,   799,   803,   806,   810,   817,   819,   822,   827,   833,
     839,   845
};

/* YYRHS -- A `-1'-separated list of the rules' RHS.  */
static const yytype_int16 yyrhs[] =
{
     101,     0,    -1,    -1,   102,    -1,   103,    -1,   102,   103,
      -1,    58,   145,    -1,   117,    -1,   135,    -1,   122,    -1,
     136,    -1,   137,    -1,   128,    -1,   116,    -1,   104,    -1,
     109,    -1,   110,    -1,   111,    -1,   112,    -1,   113,    -1,
     114,    -1,   119,    -1,   140,    -1,   141,    -1,   142,    -1,
     138,    -1,   115,    -1,   132,    -1,   133,    -1,   139,    -1,
       8,     6,   105,   107,   106,   143,    -1,    -1,    11,   162,
      -1,    11,     6,    -1,    -1,    13,   108,    -1,    -1,    12,
     108,    -1,     6,    -1,   162,    -1,   108,    59,     6,    -1,
     108,    59,   162,    -1,    58,    17,    56,     6,    57,    -1,
      17,    56,     6,    57,    52,    53,   143,    -1,    58,    18,
      56,     6,    57,    -1,    18,    56,     6,    57,    52,     6,
      53,   143,    -1,    58,    19,    56,     6,    57,    -1,    58,
      16,    56,     6,    57,    -1,    58,    16,    56,    34,    44,
       6,    57,    -1,    58,    15,    56,     6,    57,    -1,    58,
      15,    56,    34,    44,     6,    57,    -1,    58,    14,    56,
       6,    57,    -1,    58,    14,    56,    34,    44,     6,    57,
      -1,    10,     6,   106,   143,    -1,     9,     6,   107,   143,
      -1,   118,   143,    -1,   118,   143,    35,   143,    36,   143,
      -1,     7,     6,    52,    53,    -1,     7,     6,    52,   165,
      53,    -1,     7,     6,    52,   165,    59,    66,     6,    53,
      -1,     7,     6,    52,    66,     6,    53,    -1,     7,    34,
      44,     6,    52,    53,    -1,     7,    34,    44,     6,    52,
     165,    53,    -1,     7,    34,    44,     6,    52,   165,    59,
      66,     6,    53,    -1,     7,    34,    44,     6,    52,    66,
       6,    53,    -1,   120,   143,    -1,     7,   121,    52,    53,
      -1,     7,   121,    52,   165,    53,    -1,    56,    57,    -1,
      56,    57,    63,    -1,    60,    -1,    61,    -1,    62,    -1,
      83,    -1,    84,    -1,    85,    -1,    86,    -1,    87,    -1,
      88,    -1,    64,    -1,    65,    -1,    66,    -1,    67,    -1,
      68,    -1,    69,    -1,    70,    -1,    71,    -1,    72,    -1,
      73,    -1,    82,    -1,    89,    -1,    90,    -1,    91,    -1,
     123,    -1,   126,    -1,    20,    52,   145,    53,   143,    -1,
      20,    52,   145,    53,   143,    21,   143,    -1,    20,    52,
     145,    53,   143,   124,    -1,    20,    52,   145,    53,   143,
     124,    21,   143,    -1,   125,    -1,   124,   125,    -1,    22,
      52,   145,    53,   143,    -1,    20,    52,   145,    59,   145,
     127,    53,   143,    -1,    -1,    59,     6,    -1,    25,    52,
     145,    53,   129,    -1,    54,   130,    55,    -1,    54,   130,
      21,   143,    55,    -1,   131,    -1,   130,   131,    -1,    26,
      52,   166,    53,   143,    -1,    23,    52,     6,    24,   145,
      53,   143,    -1,    23,    52,    52,     6,    59,     6,    53,
      24,   145,    53,   143,    -1,    37,   143,    38,    52,     6,
      53,   143,   134,    -1,    -1,    39,    54,   144,    55,    -1,
      58,    32,    -1,    58,    32,   145,    -1,    58,    30,    -1,
      58,    31,    -1,    58,     7,     6,    52,    53,    -1,    58,
       7,     6,    52,   165,    53,    -1,    58,     7,     6,    52,
     165,    59,    66,     6,    53,    -1,    58,     7,     6,    52,
      66,     6,    53,    -1,    58,    40,    52,   145,    53,    -1,
      58,    33,    -1,    58,    27,    52,    53,    -1,    58,    27,
      52,   166,    53,    -1,    58,    29,    52,    53,    -1,    58,
      29,    52,   166,    53,    -1,    54,    55,    -1,    54,   144,
      55,    -1,   103,    -1,   144,   103,    -1,   147,    -1,   159,
     146,   145,    -1,    63,    -1,    74,    -1,    75,    -1,    76,
      -1,    77,    -1,    78,    -1,    79,    -1,    80,    -1,    81,
      -1,    92,    -1,    93,    -1,    94,    -1,    95,    -1,   148,
      -1,   147,    61,   148,    -1,   149,    -1,   148,    60,   149,
      -1,   150,    -1,   149,    71,   150,    -1,   151,    -1,   150,
      72,   151,    -1,   152,    -1,   151,    70,   152,    -1,   153,
      -1,   152,    83,   153,    -1,   152,    84,   153,    -1,   154,
      -1,   153,    85,   154,    -1,   153,    86,   154,    -1,   153,
      87,   154,    -1,   153,    88,   154,    -1,   155,    -1,   154,
      82,   155,    -1,   154,    89,   155,    -1,   154,    90,   155,
      -1,   154,    91,   155,    -1,   156,    -1,   155,    64,   156,
      -1,   155,    65,   156,    -1,   157,    -1,   156,    66,   157,
      -1,   156,    67,   157,    -1,   156,    68,   157,    -1,   158,
      -1,   157,    69,   158,    -1,   159,    -1,    62,   158,    -1,
      73,   158,    -1,    65,   158,    -1,    64,   158,    -1,   161,
      -1,   159,    56,   145,    57,    -1,   159,    44,     6,    52,
      53,   160,    -1,   159,    44,     6,    52,   166,    53,   160,
      -1,     6,    52,    53,   160,    -1,     6,    52,   166,    53,
     160,    -1,   159,    44,     6,    -1,    -1,    54,    55,    -1,
      54,   144,    55,    -1,    54,    28,    96,    56,   165,    57,
      99,   144,    55,    -1,   162,    -1,    52,   145,    53,    -1,
       6,    -1,     3,    -1,     4,    -1,     5,    -1,    41,    -1,
      42,    -1,    43,    -1,    34,    -1,   163,    -1,   167,    -1,
     170,    -1,   164,     6,    -1,    97,   164,     6,    -1,    97,
       6,    -1,    56,    57,    -1,    56,   166,    57,    -1,    56,
     166,    59,    57,    -1,     6,    97,    -1,   164,     6,    97,
      -1,     6,    -1,   165,    59,     6,    -1,   145,    -1,   166,
      59,   145,    -1,    54,    55,    -1,    54,   168,    55,    -1,
      54,   168,   145,    96,   145,    55,    -1,   169,    -1,   168,
     169,    -1,   145,    96,   145,    59,    -1,    52,   145,    96,
     145,    53,    -1,    52,   145,    96,   145,    57,    -1,    56,
     145,    96,   145,    57,    -1,    56,   145,    96,   145,    53,
      -1
};

/* YYRLINE[YYN] -- source line where rule number YYN was defined.  */
static const yytype_uint16 yyrline[] =
{
       0,   106,   106,   108,   112,   115,   121,   126,   127,   128,
     129,   130,   131,   132,   133,   134,   135,   136,   137,   138,
     139,   140,   141,   142,   143,   144,   145,   146,   147,   148,
     153,   161,   164,   167,   173,   176,   182,   185,   191,   196,
     201,   205,   213,   218,   227,   232,   241,   250,   255,   263,
     268,   276,   281,   289,   298,   307,   312,   320,   323,   326,
     329,   332,   335,   338,   341,   347,   355,   358,   369,   372,
     375,   378,   381,   384,   387,   390,   393,   396,   399,   402,
     405,   408,   411,   414,   417,   420,   423,   426,   429,   432,
     435,   438,   441,   448,   452,   459,   463,   468,   472,   479,
     484,   491,   498,   505,   508,   515,   523,   527,   534,   539,
     546,   554,   559,   568,   576,   579,   587,   592,   601,   609,
     618,   623,   628,   633,   642,   651,   660,   665,   674,   679,
     687,   691,   697,   702,   710,   711,   718,   721,   724,   727,
     730,   733,   736,   739,   742,   745,   748,   751,   754,   760,
     761,   768,   769,   776,   777,   784,   785,   792,   793,   800,
     801,   805,   812,   813,   817,   821,   825,   832,   833,   837,
     841,   845,   852,   853,   857,   864,   865,   869,   873,   880,
     881,   888,   889,   893,   897,   901,   908,   909,   913,   917,
     921,   925,   929,   936,   939,   942,   945,   951,   952,   955,
     959,   963,   967,   971,   975,   979,   983,   987,   988,   989,
     993,   997,  1001,  1008,  1012,  1016,  1024,  1029,  1037,  1042,
    1049,  1054,  1061,  1065,  1069,  1076,  1081,  1087,  1092,  1096,
    1100,  1104
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
  "ITER", "FILED", "SHARP", "CONLON", "$accept", "start",
  "translation_unit", "statement", "class_statement", "extends_field",
  "interface_field", "module_field", "field_identifier_list",
  "getter_statement", "setter_statement", "getter_setter_statement",
  "everyone_statement", "relative_statement", "personal_statement",
  "interface_statement", "module_statement", "function_statement",
  "function_header", "function_operator_statement",
  "function_operator_header", "operator", "if_statement", "condition_if",
  "elseif_list", "elseif", "loop_if", "loop_time", "switch_statement",
  "switch_block", "when_list", "when", "for_statement", "order_statement",
  "ignore_block", "return_statement", "break_statement",
  "continue_statement", "interface_func_statement", "groan_statement",
  "block_statement", "cast_statement", "super_statement", "block",
  "statement_list", "expression", "assign_symbol", "logic_or_expression",
  "logic_and_expression", "logic_bit_or_expression",
  "logic_bit_xor_expression", "logic_bit_and_expression",
  "logic_equal_compare_expression", "logic_not_equal_expression",
  "logic_shift_expression", "add_sub_expression", "mul_div_mod_expression",
  "power_expression", "unary_expression", "top_expression",
  "closure_block", "primary_expression", "identifier_with_field",
  "array_literal", "field_identifier", "identifier_list",
  "expression_list", "hash_literal", "hash_pair_list", "hash_pair",
  "range_literal", YY_NULL
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
       0,   100,   101,   101,   102,   102,   103,   103,   103,   103,
     103,   103,   103,   103,   103,   103,   103,   103,   103,   103,
     103,   103,   103,   103,   103,   103,   103,   103,   103,   103,
     104,   105,   105,   105,   106,   106,   107,   107,   108,   108,
     108,   108,   109,   109,   110,   110,   111,   112,   112,   113,
     113,   114,   114,   115,   116,   117,   117,   118,   118,   118,
     118,   118,   118,   118,   118,   119,   120,   120,   121,   121,
     121,   121,   121,   121,   121,   121,   121,   121,   121,   121,
     121,   121,   121,   121,   121,   121,   121,   121,   121,   121,
     121,   121,   121,   122,   122,   123,   123,   123,   123,   124,
     124,   125,   126,   127,   127,   128,   129,   129,   130,   130,
     131,   132,   132,   133,   134,   134,   135,   135,   136,   137,
     138,   138,   138,   138,   139,   140,   141,   141,   142,   142,
     143,   143,   144,   144,   145,   145,   146,   146,   146,   146,
     146,   146,   146,   146,   146,   146,   146,   146,   146,   147,
     147,   148,   148,   149,   149,   150,   150,   151,   151,   152,
     152,   152,   153,   153,   153,   153,   153,   154,   154,   154,
     154,   154,   155,   155,   155,   156,   156,   156,   156,   157,
     157,   158,   158,   158,   158,   158,   159,   159,   159,   159,
     159,   159,   159,   160,   160,   160,   160,   161,   161,   161,
     161,   161,   161,   161,   161,   161,   161,   161,   161,   161,
     162,   162,   162,   163,   163,   163,   164,   164,   165,   165,
     166,   166,   167,   167,   167,   168,   168,   169,   170,   170,
     170,   170
};

/* YYR2[YYN] -- Number of symbols composing right hand side of rule YYN.  */
static const yytype_uint8 yyr2[] =
{
       0,     2,     0,     1,     1,     2,     2,     1,     1,     1,
       1,     1,     1,     1,     1,     1,     1,     1,     1,     1,
       1,     1,     1,     1,     1,     1,     1,     1,     1,     1,
       6,     0,     2,     2,     0,     2,     0,     2,     1,     1,
       3,     3,     5,     7,     5,     8,     5,     5,     7,     5,
       7,     5,     7,     4,     4,     2,     6,     4,     5,     8,
       6,     6,     7,    10,     8,     2,     4,     5,     2,     3,
       1,     1,     1,     1,     1,     1,     1,     1,     1,     1,
       1,     1,     1,     1,     1,     1,     1,     1,     1,     1,
       1,     1,     1,     1,     1,     5,     7,     6,     8,     1,
       2,     5,     8,     0,     2,     5,     3,     5,     1,     2,
       5,     7,    11,     8,     0,     4,     2,     3,     2,     2,
       5,     6,     9,     7,     5,     2,     4,     5,     4,     5,
       2,     3,     1,     2,     1,     3,     1,     1,     1,     1,
       1,     1,     1,     1,     1,     1,     1,     1,     1,     1,
       3,     1,     3,     1,     3,     1,     3,     1,     3,     1,
       3,     3,     1,     3,     3,     3,     3,     1,     3,     3,
       3,     3,     1,     3,     3,     1,     3,     3,     3,     1,
       3,     1,     2,     2,     2,     2,     1,     4,     6,     7,
       4,     5,     3,     0,     2,     3,     9,     1,     3,     1,
       1,     1,     1,     1,     1,     1,     1,     1,     1,     1,
       2,     3,     2,     2,     3,     4,     2,     3,     1,     3,
       1,     3,     2,     3,     6,     1,     2,     4,     5,     5,
       5,     5
};

/* YYDEFACT[STATE-NAME] -- Default reduction number in state STATE-NUM.
   Performed when YYTABLE doesn't specify something else to do.  Zero
   means the default is an error.  */
static const yytype_uint8 yydefact[] =
{
       2,     0,     0,     0,     0,     0,     0,     0,     0,     0,
       0,     0,     0,     3,     4,    14,    15,    16,    17,    18,
      19,    20,    26,    13,     7,     0,    21,     0,     9,    93,
      94,    12,    27,    28,     8,    10,    11,    25,    29,    22,
      23,    24,     0,     0,     0,    70,    71,    72,    79,    80,
      81,    82,    83,    84,    85,    86,    87,    88,    89,    73,
      74,    75,    76,    77,    78,    90,    91,    92,     0,    31,
      36,    34,     0,     0,     0,     0,     0,     0,     0,   200,
     201,   202,   199,     0,     0,     0,     0,     0,     0,     0,
       0,     0,   118,   119,   116,   125,   206,     0,   203,   204,
     205,     0,     0,     0,     0,     0,     0,     0,     0,     6,
     134,   149,   151,   153,   155,   157,   159,   162,   167,   172,
     175,   179,   181,   186,   197,   207,     0,   208,   209,     1,
       5,    55,    65,     0,     0,    68,     0,     0,    36,     0,
       0,     0,     0,     0,     0,     0,     0,     0,     0,   130,
     132,     0,     0,     0,   216,     0,     0,     0,     0,     0,
       0,     0,     0,     0,   117,     0,     0,   222,     0,     0,
     225,   213,   220,     0,   182,   181,   185,   184,   183,   212,
       0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
       0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
       0,     0,     0,     0,   136,   137,   138,   139,   140,   141,
     142,   143,   144,   145,   146,   147,   148,     0,   210,     0,
     218,    57,     0,     0,     0,    69,    66,     0,    33,    32,
      34,    38,    37,    39,    54,    35,    53,     0,     0,     0,
       0,     0,     0,     0,   131,   133,     0,   193,   220,     0,
       0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
     126,     0,   128,     0,     0,   198,     0,     0,   223,     0,
     226,     0,   214,     0,   211,   150,   152,   154,   156,   158,
     160,   161,   163,   164,   165,   166,   168,   169,   170,   171,
     173,   174,   176,   177,   178,   180,   192,     0,   135,   217,
       0,     0,    58,     0,     0,    67,     0,     0,     0,     0,
       0,    95,   103,     0,     0,     0,   105,     0,     0,   190,
     193,     0,   120,     0,     0,    51,     0,    49,     0,    47,
       0,    42,    44,    46,   127,   129,   124,     0,     0,     0,
       0,   215,   221,     0,   187,     0,    60,   219,     0,    61,
       0,     0,    30,    40,    41,     0,     0,     0,     0,    97,
      99,     0,     0,     0,     0,     0,     0,   108,     0,     0,
     194,     0,   191,     0,   121,     0,     0,     0,     0,   228,
     229,   227,     0,   231,   230,   193,     0,    56,     0,     0,
      62,     0,    43,     0,    96,     0,     0,   100,   104,     0,
     111,     0,     0,     0,   106,   109,   114,     0,   195,   123,
       0,    52,    50,    48,   224,   188,   193,    59,    64,     0,
      45,     0,    98,   102,     0,     0,     0,     0,   113,     0,
       0,   189,     0,     0,     0,     0,   107,     0,     0,   122,
      63,   101,     0,   110,     0,     0,   112,   115,     0,     0,
     196
};

/* YYDEFGOTO[NTERM-NUM].  */
static const yytype_int16 yydefgoto[] =
{
      -1,    12,    13,   150,    15,   138,   142,   140,   232,    16,
      17,    18,    19,    20,    21,    22,    23,    24,    25,    26,
      27,    68,    28,    29,   359,   360,    30,   362,    31,   316,
     366,   367,    32,    33,   428,    34,    35,    36,    37,    38,
      39,    40,    41,    78,   151,   248,   217,   110,   111,   112,
     113,   114,   115,   116,   117,   118,   119,   120,   121,   122,
     319,   123,   124,   125,   126,   223,   173,   127,   169,   170,
     128
};

/* YYPACT[STATE-NUM] -- Index in YYTABLE of the portion describing
   STATE-NUM.  */
#define YYPACT_NINF -315
static const yytype_int16 yypact[] =
{
     820,   656,    36,    38,    47,    15,    33,    39,   139,   140,
      71,   207,    83,   820,  -315,  -315,  -315,  -315,  -315,  -315,
    -315,  -315,  -315,  -315,  -315,    71,  -315,    71,  -315,  -315,
    -315,  -315,  -315,  -315,  -315,  -315,  -315,  -315,  -315,  -315,
    -315,  -315,   141,    98,   142,  -315,  -315,  -315,  -315,  -315,
    -315,  -315,  -315,  -315,  -315,  -315,  -315,  -315,  -315,  -315,
    -315,  -315,  -315,  -315,  -315,  -315,  -315,  -315,   145,   117,
     186,   187,   195,   196,   637,    21,   637,   340,   165,  -315,
    -315,  -315,   -18,   198,   149,   150,   151,   160,   161,   162,
     157,   167,  -315,  -315,   637,  -315,  -315,   168,  -315,  -315,
    -315,   637,   273,   348,   637,   637,   637,   637,   221,  -315,
     169,   173,   158,   156,   172,    56,    67,   -43,    82,    -5,
     166,  -315,    87,  -315,  -315,  -315,   237,  -315,  -315,  -315,
    -315,   209,  -315,     2,   239,   183,    22,    24,   186,    25,
      71,    25,    71,   194,   197,   -10,   228,   247,   203,  -315,
    -315,   741,   208,   419,  -315,   213,    26,    30,    32,   260,
     261,   262,   424,   490,  -315,   637,   -16,  -315,   174,   495,
    -315,  -315,   177,    19,  -315,   -15,  -315,  -315,  -315,   178,
     268,   637,   637,   637,   637,   637,   637,   637,   637,   637,
     637,   637,   637,   637,   637,   637,   637,   637,   637,   637,
     637,   637,   275,   637,  -315,  -315,  -315,  -315,  -315,  -315,
    -315,  -315,  -315,  -315,  -315,  -315,  -315,   637,   185,    71,
    -315,  -315,   278,    -8,   233,  -315,  -315,    -3,   178,  -315,
     187,   178,   227,  -315,  -315,   227,  -315,   235,   236,    71,
     637,   637,   230,   242,  -315,  -315,   284,   243,  -315,    28,
       4,   234,   255,   244,   256,   249,   265,   254,   263,   266,
    -315,    31,  -315,    73,   264,  -315,   637,   637,  -315,   222,
    -315,   637,  -315,   561,   185,   173,   158,   156,   172,    56,
      67,    67,   -43,   -43,   -43,   -43,    82,    82,    82,    82,
      -5,    -5,   166,   166,   166,  -315,   267,   269,  -315,  -315,
     288,   279,  -315,     8,     6,  -315,   328,    71,    27,   283,
     335,   153,   286,   289,   349,   330,  -315,   306,   285,  -315,
     243,   637,  -315,   355,    74,  -315,   356,  -315,   358,  -315,
     360,  -315,  -315,  -315,  -315,  -315,  -315,     1,   308,   637,
      84,  -315,  -315,   595,  -315,    71,  -315,  -315,   362,  -315,
     367,    76,  -315,   178,  -315,    71,   321,    71,   324,   155,
    -315,   372,   327,    71,   331,   329,    14,  -315,    71,   287,
    -315,   745,  -315,   332,  -315,    11,   336,   337,   339,  -315,
    -315,  -315,    90,  -315,  -315,   243,    77,  -315,   334,   344,
    -315,    16,  -315,    71,  -315,   637,    71,  -315,  -315,    71,
    -315,   364,   637,    71,  -315,  -315,   353,   343,  -315,  -315,
     395,  -315,  -315,  -315,  -315,  -315,   243,  -315,  -315,   397,
    -315,   354,  -315,  -315,   637,    85,   351,   363,  -315,   402,
     361,  -315,   366,    71,   373,    71,  -315,   820,    29,  -315,
    -315,  -315,    71,  -315,   767,   317,  -315,  -315,   820,   798,
    -315
};

/* YYPGOTO[NTERM-NUM].  */
static const yytype_int16 yypgoto[] =
{
    -315,  -315,  -315,     0,  -315,  -315,   190,   293,   291,  -315,
    -315,  -315,  -315,  -315,  -315,  -315,  -315,  -315,  -315,  -315,
    -315,  -315,  -315,  -315,  -315,    75,  -315,  -315,  -315,  -315,
    -315,    69,  -315,  -315,  -315,  -315,  -315,  -315,  -315,  -315,
    -315,  -315,  -315,   -24,  -314,    -9,  -315,  -315,   252,   257,
     253,   258,   270,     3,   -19,    -7,   -13,   -41,   -81,   -86,
    -268,  -315,  -130,  -315,   333,  -131,  -147,  -315,  -315,   271,
    -315
};

/* YYTABLE[YYPACT[STATE-NUM]].  What to do in state STATE-NUM.  If
   positive, shift that token.  If negative, reduce the rule which
   number is the opposite.  If YYTABLE_NINF, syntax error.  */
#define YYTABLE_NINF -1
static const yytype_uint16 yytable[] =
{
      14,   131,   109,   132,   371,   227,   249,   229,   220,   233,
     220,   233,   220,   130,   347,   261,   263,   347,   175,   175,
     175,   175,   347,   174,   176,   177,   178,   146,   220,   202,
     228,   231,   251,   353,   153,   403,   253,   265,   255,   192,
     365,   203,    69,   239,    70,   302,   193,   194,   195,   240,
     305,   303,   372,    71,   379,   221,   306,   322,   380,   349,
     252,   198,   199,   200,   254,   145,   256,   148,   222,   404,
     323,    72,   350,   147,   348,   226,   272,   410,   273,   154,
     266,   320,   419,   129,   334,   164,   445,   321,   306,    73,
     321,    74,   166,   168,   172,   175,   175,   175,   175,   175,
     175,   175,   175,   175,   175,   175,   175,   175,   175,   175,
     175,   175,   175,   175,   175,   175,   234,   415,   236,   324,
     295,   108,   108,   444,   108,    77,   335,   374,   137,   390,
     416,   202,   321,   375,   449,   391,   321,   383,   435,   186,
     187,   384,   134,   203,   321,   414,   196,   197,   431,   381,
     204,   245,   188,   189,   190,   191,   264,   292,   293,   294,
     269,   205,   206,   207,   208,   209,   210,   211,   212,   282,
     283,   284,   285,   351,   357,   358,   396,   358,   354,   213,
     214,   215,   216,   290,   291,   286,   287,   288,   289,   280,
     281,    75,    76,   133,   297,   300,   386,   136,   139,   135,
     141,   143,   144,   152,   155,   156,   157,   158,   298,   162,
      79,    80,    81,    82,    83,   311,   159,   160,   161,   163,
     165,    84,    85,    86,    87,    88,    89,   179,   184,   183,
     181,   312,   313,   182,    90,   201,    91,    92,    93,    94,
      95,    96,   185,   218,   219,   224,   225,    97,    98,    99,
     100,   237,   241,   242,   238,   425,   243,   337,   338,   101,
     246,   102,   340,   103,   342,   250,   257,   258,   259,   104,
     267,   105,   106,   271,   274,   154,    79,    80,    81,    82,
     107,   296,   299,   352,   301,   304,   308,   309,   310,   314,
     317,   325,     1,     2,     3,     4,   315,   318,   438,   326,
     328,   327,     5,     6,   108,     7,   329,    96,     8,   330,
       9,   331,   342,   369,    98,    99,   100,   336,   339,   343,
     332,   387,    10,   333,   345,   101,   344,   102,   167,   103,
     382,   392,   346,   394,   347,   104,   355,   105,   106,   400,
     370,   356,   363,    11,   406,   361,   107,     1,     2,     3,
       4,    79,    80,    81,    82,   364,   365,     5,     6,   368,
       7,   373,   376,     8,   377,     9,   378,   381,   388,   420,
     108,   245,   422,   389,   393,   423,   395,    10,   398,   426,
     399,   402,    96,   407,   401,   409,   421,   417,   424,    98,
      99,   100,   427,   411,   412,   149,   413,   418,    11,   429,
     101,   430,   102,   432,   103,   171,   436,   433,   220,   441,
     104,   443,   105,   106,   439,   434,   448,   437,   446,   440,
     307,   107,    79,    80,    81,    82,   442,    79,    80,    81,
      82,   230,   235,   275,   397,   405,   277,     0,     0,   276,
     270,   180,   278,     0,   245,   108,     0,     0,     0,   245,
       0,     0,     0,    96,     0,   279,     0,     0,    96,     0,
      98,    99,   100,     0,     0,    98,    99,   100,     0,     0,
       0,   101,   247,   102,     0,   103,   101,   260,   102,     0,
     103,   104,     0,   105,   106,     0,   104,     0,   105,   106,
       0,     0,   107,    79,    80,    81,    82,   107,    79,    80,
      81,    82,     0,     0,     0,     0,     0,     0,     0,     0,
       0,     0,     0,     0,     0,     0,   108,     0,     0,     0,
       0,   108,     0,     0,    96,     0,     0,     0,     0,    96,
       0,    98,    99,   100,     0,     0,    98,    99,   100,     0,
       0,     0,   101,   262,   102,     0,   103,   101,     0,   102,
     268,   103,   104,     0,   105,   106,     0,   104,     0,   105,
     106,     0,     0,   107,    79,    80,    81,    82,   107,     0,
       0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
       0,     0,     0,     0,     0,     0,     0,   108,     0,     0,
       0,     0,   108,     0,     0,    96,     0,     0,    79,    80,
      81,    82,    98,    99,   100,     0,     0,     0,     0,     0,
       0,     0,     0,   101,     0,   102,     0,   103,   341,     0,
       0,     0,     0,   104,     0,   105,   106,     0,     0,    96,
       0,     0,     0,     0,   107,     0,    98,    99,   100,     0,
      79,    80,    81,    82,     0,     0,     0,   101,   385,   102,
       0,   103,     0,     0,     0,     0,     0,   104,   108,   105,
     106,     0,    42,     0,     0,     0,     0,     0,   107,     0,
       0,    96,     0,     0,     0,     0,     0,     0,    98,    99,
     100,     0,     0,     0,     0,     0,     0,     0,     0,   101,
      43,   102,   108,   103,     0,     0,     0,     0,     0,   104,
       0,   105,   106,     0,     0,     0,     0,     0,     0,     0,
     107,     0,    44,     0,     0,     0,    45,    46,    47,     0,
      48,    49,    50,    51,    52,    53,    54,    55,    56,    57,
       0,     0,     0,     0,   108,     0,     0,     0,    58,    59,
      60,    61,    62,    63,    64,    65,    66,    67,     1,     2,
       3,     4,     1,     2,     3,     4,     0,     0,     5,     6,
       0,     7,     5,     6,     8,     7,     9,     0,     8,     0,
       9,     0,     0,     0,     1,     2,     3,     4,    10,     0,
       0,     0,    10,     0,     5,     6,     0,     7,     0,     0,
       8,     0,     9,     0,     0,     0,   244,     0,     0,    11,
     408,     0,     0,    11,    10,     1,     2,     3,     4,     0,
       0,     0,     0,     0,     0,     5,     6,     0,     7,     0,
       0,     8,   447,     9,     0,    11,     0,     1,     2,     3,
       4,     0,     0,     0,     0,    10,     0,     5,     6,     0,
       7,     0,     0,     8,     0,     9,     0,     0,     0,     0,
       0,     0,     0,   450,     0,     0,    11,    10,     0,     0,
       0,     0,     0,     0,     0,     0,     0,     0,     0,     0,
       0,     0,     0,     0,     0,     0,     0,     0,    11
};

#define yypact_value_is_default(Yystate) \
  (!!((Yystate) == (-315)))

#define yytable_value_is_error(Yytable_value) \
  YYID (0)

static const yytype_int16 yycheck[] =
{
       0,    25,    11,    27,   318,   136,   153,   137,     6,   139,
       6,   141,     6,    13,     6,   162,   163,     6,   104,   105,
     106,   107,     6,   104,   105,   106,   107,     6,     6,    44,
       6,     6,     6,     6,    52,    21,     6,    53,     6,    82,
      26,    56,     6,    53,     6,    53,    89,    90,    91,    59,
      53,    59,   320,     6,    53,    53,    59,    53,    57,    53,
      34,    66,    67,    68,    34,    74,    34,    76,    66,    55,
      66,    56,    66,    52,    66,    53,    57,    66,    59,    97,
      96,    53,    66,     0,    53,    94,    57,    59,    59,    56,
      59,    52,   101,   102,   103,   181,   182,   183,   184,   185,
     186,   187,   188,   189,   190,   191,   192,   193,   194,   195,
     196,   197,   198,   199,   200,   201,   140,   385,   142,   250,
     201,    97,    97,   437,    97,    54,    53,    53,    11,    53,
      53,    44,    59,    59,   448,    59,    59,    53,    53,    83,
      84,    57,    44,    56,    59,    55,    64,    65,   416,    59,
      63,   151,    85,    86,    87,    88,   165,   198,   199,   200,
     169,    74,    75,    76,    77,    78,    79,    80,    81,   188,
     189,   190,   191,   304,    21,    22,    21,    22,   308,    92,
      93,    94,    95,   196,   197,   192,   193,   194,   195,   186,
     187,    52,    52,    52,   203,   219,   343,    52,    12,    57,
      13,     6,     6,    38,     6,    56,    56,    56,   217,    52,
       3,     4,     5,     6,     7,   239,    56,    56,    56,    52,
      52,    14,    15,    16,    17,    18,    19,     6,    72,    71,
      61,   240,   241,    60,    27,    69,    29,    30,    31,    32,
      33,    34,    70,     6,    35,     6,    63,    40,    41,    42,
      43,    57,    24,     6,    57,   402,    53,   266,   267,    52,
      52,    54,   271,    56,   273,    52,     6,     6,     6,    62,
      96,    64,    65,    96,     6,    97,     3,     4,     5,     6,
      73,     6,    97,   307,     6,    52,    59,    52,    52,    59,
       6,    57,     7,     8,     9,    10,    54,    54,   429,    44,
      44,    57,    17,    18,    97,    20,    57,    34,    23,    44,
      25,    57,   321,    28,    41,    42,    43,    53,    96,    52,
      57,   345,    37,    57,    36,    52,    57,    54,    55,    56,
     339,   355,    53,   357,     6,    62,    53,    64,    65,   363,
      55,     6,    53,    58,   368,    59,    73,     7,     8,     9,
      10,     3,     4,     5,     6,     6,    26,    17,    18,    53,
      20,     6,     6,    23,     6,    25,     6,    59,     6,   393,
      97,   371,   396,     6,    53,   399,    52,    37,     6,   403,
      53,    52,    34,    96,    53,    53,   395,    53,    24,    41,
      42,    43,    39,    57,    57,    55,    57,    53,    58,    56,
      52,     6,    54,     6,    56,    57,    55,    53,     6,   433,
      62,   435,    64,    65,    53,   424,    99,    54,   442,    53,
     230,    73,     3,     4,     5,     6,    53,     3,     4,     5,
       6,   138,   141,   181,   359,   366,   183,    -1,    -1,   182,
     169,   108,   184,    -1,   444,    97,    -1,    -1,    -1,   449,
      -1,    -1,    -1,    34,    -1,   185,    -1,    -1,    34,    -1,
      41,    42,    43,    -1,    -1,    41,    42,    43,    -1,    -1,
      -1,    52,    53,    54,    -1,    56,    52,    53,    54,    -1,
      56,    62,    -1,    64,    65,    -1,    62,    -1,    64,    65,
      -1,    -1,    73,     3,     4,     5,     6,    73,     3,     4,
       5,     6,    -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,
      -1,    -1,    -1,    -1,    -1,    -1,    97,    -1,    -1,    -1,
      -1,    97,    -1,    -1,    34,    -1,    -1,    -1,    -1,    34,
      -1,    41,    42,    43,    -1,    -1,    41,    42,    43,    -1,
      -1,    -1,    52,    53,    54,    -1,    56,    52,    -1,    54,
      55,    56,    62,    -1,    64,    65,    -1,    62,    -1,    64,
      65,    -1,    -1,    73,     3,     4,     5,     6,    73,    -1,
      -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,
      -1,    -1,    -1,    -1,    -1,    -1,    -1,    97,    -1,    -1,
      -1,    -1,    97,    -1,    -1,    34,    -1,    -1,     3,     4,
       5,     6,    41,    42,    43,    -1,    -1,    -1,    -1,    -1,
      -1,    -1,    -1,    52,    -1,    54,    -1,    56,    57,    -1,
      -1,    -1,    -1,    62,    -1,    64,    65,    -1,    -1,    34,
      -1,    -1,    -1,    -1,    73,    -1,    41,    42,    43,    -1,
       3,     4,     5,     6,    -1,    -1,    -1,    52,    53,    54,
      -1,    56,    -1,    -1,    -1,    -1,    -1,    62,    97,    64,
      65,    -1,     6,    -1,    -1,    -1,    -1,    -1,    73,    -1,
      -1,    34,    -1,    -1,    -1,    -1,    -1,    -1,    41,    42,
      43,    -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,    52,
      34,    54,    97,    56,    -1,    -1,    -1,    -1,    -1,    62,
      -1,    64,    65,    -1,    -1,    -1,    -1,    -1,    -1,    -1,
      73,    -1,    56,    -1,    -1,    -1,    60,    61,    62,    -1,
      64,    65,    66,    67,    68,    69,    70,    71,    72,    73,
      -1,    -1,    -1,    -1,    97,    -1,    -1,    -1,    82,    83,
      84,    85,    86,    87,    88,    89,    90,    91,     7,     8,
       9,    10,     7,     8,     9,    10,    -1,    -1,    17,    18,
      -1,    20,    17,    18,    23,    20,    25,    -1,    23,    -1,
      25,    -1,    -1,    -1,     7,     8,     9,    10,    37,    -1,
      -1,    -1,    37,    -1,    17,    18,    -1,    20,    -1,    -1,
      23,    -1,    25,    -1,    -1,    -1,    55,    -1,    -1,    58,
      55,    -1,    -1,    58,    37,     7,     8,     9,    10,    -1,
      -1,    -1,    -1,    -1,    -1,    17,    18,    -1,    20,    -1,
      -1,    23,    55,    25,    -1,    58,    -1,     7,     8,     9,
      10,    -1,    -1,    -1,    -1,    37,    -1,    17,    18,    -1,
      20,    -1,    -1,    23,    -1,    25,    -1,    -1,    -1,    -1,
      -1,    -1,    -1,    55,    -1,    -1,    58,    37,    -1,    -1,
      -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,
      -1,    -1,    -1,    -1,    -1,    -1,    -1,    -1,    58
};

/* YYSTOS[STATE-NUM] -- The (internal number of the) accessing
   symbol of state STATE-NUM.  */
static const yytype_uint8 yystos[] =
{
       0,     7,     8,     9,    10,    17,    18,    20,    23,    25,
      37,    58,   101,   102,   103,   104,   109,   110,   111,   112,
     113,   114,   115,   116,   117,   118,   119,   120,   122,   123,
     126,   128,   132,   133,   135,   136,   137,   138,   139,   140,
     141,   142,     6,    34,    56,    60,    61,    62,    64,    65,
      66,    67,    68,    69,    70,    71,    72,    73,    82,    83,
      84,    85,    86,    87,    88,    89,    90,    91,   121,     6,
       6,     6,    56,    56,    52,    52,    52,    54,   143,     3,
       4,     5,     6,     7,    14,    15,    16,    17,    18,    19,
      27,    29,    30,    31,    32,    33,    34,    40,    41,    42,
      43,    52,    54,    56,    62,    64,    65,    73,    97,   145,
     147,   148,   149,   150,   151,   152,   153,   154,   155,   156,
     157,   158,   159,   161,   162,   163,   164,   167,   170,     0,
     103,   143,   143,    52,    44,    57,    52,    11,   105,    12,
     107,    13,   106,     6,     6,   145,     6,    52,   145,    55,
     103,   144,    38,    52,    97,     6,    56,    56,    56,    56,
      56,    56,    52,    52,   145,    52,   145,    55,   145,   168,
     169,    57,   145,   166,   158,   159,   158,   158,   158,     6,
     164,    61,    60,    71,    72,    70,    83,    84,    85,    86,
      87,    88,    82,    89,    90,    91,    64,    65,    66,    67,
      68,    69,    44,    56,    63,    74,    75,    76,    77,    78,
      79,    80,    81,    92,    93,    94,    95,   146,     6,    35,
       6,    53,    66,   165,     6,    63,    53,   165,     6,   162,
     107,     6,   108,   162,   143,   108,   143,    57,    57,    53,
      59,    24,     6,    53,    55,   103,    52,    53,   145,   166,
      52,     6,    34,     6,    34,     6,    34,     6,     6,     6,
      53,   166,    53,   166,   145,    53,    96,    96,    55,   145,
     169,    96,    57,    59,     6,   148,   149,   150,   151,   152,
     153,   153,   154,   154,   154,   154,   155,   155,   155,   155,
     156,   156,   157,   157,   157,   158,     6,   145,   145,    97,
     143,     6,    53,    59,    52,    53,    59,   106,    59,    52,
      52,   143,   145,   145,    59,    54,   129,     6,    54,   160,
      53,    59,    53,    66,   165,    57,    44,    57,    44,    57,
      44,    57,    57,    57,    53,    53,    53,   145,   145,    96,
     145,    57,   145,    52,    57,    36,    53,     6,    66,    53,
      66,   165,   143,     6,   162,    53,     6,    21,    22,   124,
     125,    59,   127,    53,     6,    26,   130,   131,    53,    28,
      55,   144,   160,     6,    53,    59,     6,     6,     6,    53,
      57,    59,   145,    53,    57,    53,   166,   143,     6,     6,
      53,    59,   143,    53,   143,    52,    21,   125,     6,    53,
     143,    53,    52,    21,    55,   131,   143,    96,    55,    53,
      66,    57,    57,    57,    55,   160,    53,    53,    53,    66,
     143,   145,   143,   143,    24,   166,   143,    39,   134,    56,
       6,   160,     6,    53,   145,    53,    55,    54,   165,    53,
      53,   143,    53,   143,   144,    57,   143,    55,    99,   144,
      55
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
        case 4:
/* Line 1792 of yacc.c  */
#line 112 "iris.y"
    {
		IrisCompiler::CurrentCompiler()->AddStatement((yyvsp[(1) - (1)].m_pStatement));
	}
    break;

  case 5:
/* Line 1792 of yacc.c  */
#line 115 "iris.y"
    {
		IrisCompiler::CurrentCompiler()->AddStatement((yyvsp[(2) - (2)].m_pStatement));
	}
    break;

  case 6:
/* Line 1792 of yacc.c  */
#line 121 "iris.y"
    {
		IrisStatement* pStatement = new IrisNormalStatement((yyvsp[(2) - (2)].m_pExpression));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 30:
/* Line 1792 of yacc.c  */
#line 153 "iris.y"
    {
		IrisStatement* pStatement = new IrisClassStatement((yyvsp[(2) - (6)].m_pIdentifier), (yyvsp[(3) - (6)].m_pExpression), (yyvsp[(4) - (6)].m_pExpressionList), (yyvsp[(5) - (6)].m_pExpressionList), (yyvsp[(6) - (6)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 31:
/* Line 1792 of yacc.c  */
#line 161 "iris.y"
    {
		(yyval.m_pExpression) = nullptr;
	}
    break;

  case 32:
/* Line 1792 of yacc.c  */
#line 164 "iris.y"
    {
		(yyval.m_pExpression) = (yyvsp[(2) - (2)].m_pExpression);
	}
    break;

  case 33:
/* Line 1792 of yacc.c  */
#line 167 "iris.y"
    {
		(yyval.m_pExpression) = new IrisFieldExpression(new IrisFieldIdentifier(nullptr, (yyvsp[(2) - (2)].m_pIdentifier), false));
	}
    break;

  case 34:
/* Line 1792 of yacc.c  */
#line 173 "iris.y"
    {
		(yyval.m_pExpressionList) = nullptr;
	}
    break;

  case 35:
/* Line 1792 of yacc.c  */
#line 176 "iris.y"
    {
		(yyval.m_pExpressionList) = (yyvsp[(2) - (2)].m_pExpressionList);
	}
    break;

  case 36:
/* Line 1792 of yacc.c  */
#line 182 "iris.y"
    {
		(yyval.m_pExpressionList) = nullptr;
	}
    break;

  case 37:
/* Line 1792 of yacc.c  */
#line 185 "iris.y"
    {
		(yyval.m_pExpressionList) = (yyvsp[(2) - (2)].m_pExpressionList);
	}
    break;

  case 38:
/* Line 1792 of yacc.c  */
#line 191 "iris.y"
    {
		IrisList<IrisExpression*>* pList = new IrisList<IrisExpression*>();
		pList->Add(new IrisFieldExpression(new IrisFieldIdentifier(nullptr, (yyvsp[(1) - (1)].m_pIdentifier), false)));
		(yyval.m_pExpressionList) = pList;		
	}
    break;

  case 39:
/* Line 1792 of yacc.c  */
#line 196 "iris.y"
    {
		IrisList<IrisExpression*>* pList = new IrisList<IrisExpression*>();
		pList->Add((yyvsp[(1) - (1)].m_pExpression));
		(yyval.m_pExpressionList) = pList;
	}
    break;

  case 40:
/* Line 1792 of yacc.c  */
#line 201 "iris.y"
    {
		(yyvsp[(1) - (3)].m_pExpressionList)->Add(new IrisFieldExpression(new IrisFieldIdentifier(nullptr, (yyvsp[(3) - (3)].m_pIdentifier), false)));
		(yyval.m_pExpressionList) = (yyvsp[(1) - (3)].m_pExpressionList);
	}
    break;

  case 41:
/* Line 1792 of yacc.c  */
#line 205 "iris.y"
    {
		(yyvsp[(1) - (3)].m_pExpressionList)->Add((yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpressionList) = (yyvsp[(1) - (3)].m_pExpressionList);		
	}
    break;

  case 42:
/* Line 1792 of yacc.c  */
#line 213 "iris.y"
    {
		IrisStatement* pStatement = new IrisGetterStatement((yyvsp[(4) - (5)].m_pIdentifier), nullptr);
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 43:
/* Line 1792 of yacc.c  */
#line 218 "iris.y"
    {
		IrisStatement* pStatement = new IrisGetterStatement((yyvsp[(3) - (7)].m_pIdentifier), (yyvsp[(7) - (7)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 44:
/* Line 1792 of yacc.c  */
#line 227 "iris.y"
    {
		IrisStatement* pStatement = new IrisSetterStatement((yyvsp[(4) - (5)].m_pIdentifier), nullptr, nullptr);
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 45:
/* Line 1792 of yacc.c  */
#line 232 "iris.y"
    {
		IrisStatement* pStatement = new IrisSetterStatement((yyvsp[(3) - (8)].m_pIdentifier), (yyvsp[(6) - (8)].m_pIdentifier), (yyvsp[(8) - (8)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 46:
/* Line 1792 of yacc.c  */
#line 241 "iris.y"
    {
		IrisStatement* pStatement = new IrisGSetterStatement((yyvsp[(4) - (5)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
 		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 47:
/* Line 1792 of yacc.c  */
#line 250 "iris.y"
    {
		IrisStatement* pStatement = new IrisAuthorityStatement(IrisAuthorityTarget::InstanceMethod, IrisAuthorityType::EveryOne, (yyvsp[(4) - (5)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 48:
/* Line 1792 of yacc.c  */
#line 255 "iris.y"
    {
		IrisStatement* pStatement = new IrisAuthorityStatement(IrisAuthorityTarget::ClassMethod, IrisAuthorityType::EveryOne, (yyvsp[(6) - (7)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 49:
/* Line 1792 of yacc.c  */
#line 263 "iris.y"
    {
		IrisStatement* pStatement = new IrisAuthorityStatement(IrisAuthorityTarget::InstanceMethod, IrisAuthorityType::Relative, (yyvsp[(4) - (5)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 50:
/* Line 1792 of yacc.c  */
#line 268 "iris.y"
    {
		IrisStatement* pStatement = new IrisAuthorityStatement(IrisAuthorityTarget::ClassMethod, IrisAuthorityType::Relative, (yyvsp[(6) - (7)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 51:
/* Line 1792 of yacc.c  */
#line 276 "iris.y"
    {
		IrisStatement* pStatement = new IrisAuthorityStatement(IrisAuthorityTarget::InstanceMethod, IrisAuthorityType::Personal, (yyvsp[(4) - (5)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 52:
/* Line 1792 of yacc.c  */
#line 281 "iris.y"
    {
		IrisStatement* pStatement = new IrisAuthorityStatement(IrisAuthorityTarget::ClassMethod, IrisAuthorityType::Personal, (yyvsp[(6) - (7)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 53:
/* Line 1792 of yacc.c  */
#line 289 "iris.y"
    {
		IrisStatement* pStatement = new IrisInterfaceStatement((yyvsp[(2) - (4)].m_pIdentifier), (yyvsp[(3) - (4)].m_pExpressionList), (yyvsp[(4) - (4)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 54:
/* Line 1792 of yacc.c  */
#line 298 "iris.y"
    {
		IrisStatement* pStatement = new IrisModuleStatement((yyvsp[(2) - (4)].m_pIdentifier), (yyvsp[(3) - (4)].m_pExpressionList), nullptr, (yyvsp[(4) - (4)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());	
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 55:
/* Line 1792 of yacc.c  */
#line 307 "iris.y"
    {
		IrisStatement* pStatement = new IrisFunctionStatement((yyvsp[(1) - (2)].m_pFunctionHeader), nullptr, nullptr, (yyvsp[(2) - (2)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 56:
/* Line 1792 of yacc.c  */
#line 312 "iris.y"
    {
		IrisStatement* pStatement = new IrisFunctionStatement((yyvsp[(1) - (6)].m_pFunctionHeader), (yyvsp[(4) - (6)].m_pBlock), (yyvsp[(6) - (6)].m_pBlock), (yyvsp[(2) - (6)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 57:
/* Line 1792 of yacc.c  */
#line 320 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(2) - (4)].m_pIdentifier), nullptr, nullptr, false);
	}
    break;

  case 58:
/* Line 1792 of yacc.c  */
#line 323 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(2) - (5)].m_pIdentifier), (yyvsp[(4) - (5)].m_pIdentifierList), nullptr, false);
	}
    break;

  case 59:
/* Line 1792 of yacc.c  */
#line 326 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(2) - (8)].m_pIdentifier), (yyvsp[(4) - (8)].m_pIdentifierList), (yyvsp[(7) - (8)].m_pIdentifier), false);
	}
    break;

  case 60:
/* Line 1792 of yacc.c  */
#line 329 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(2) - (6)].m_pIdentifier), nullptr, (yyvsp[(5) - (6)].m_pIdentifier), false);
	}
    break;

  case 61:
/* Line 1792 of yacc.c  */
#line 332 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(4) - (6)].m_pIdentifier), nullptr, nullptr, true);
	}
    break;

  case 62:
/* Line 1792 of yacc.c  */
#line 335 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(4) - (7)].m_pIdentifier), (yyvsp[(6) - (7)].m_pIdentifierList), nullptr, true);
	}
    break;

  case 63:
/* Line 1792 of yacc.c  */
#line 338 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(4) - (10)].m_pIdentifier), (yyvsp[(6) - (10)].m_pIdentifierList), (yyvsp[(9) - (10)].m_pIdentifier), true);
	}
    break;

  case 64:
/* Line 1792 of yacc.c  */
#line 341 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(4) - (8)].m_pIdentifier), nullptr, (yyvsp[(7) - (8)].m_pIdentifier), true);
	}
    break;

  case 65:
/* Line 1792 of yacc.c  */
#line 347 "iris.y"
    {
		IrisStatement* pStatement = new IrisFunctionStatement((yyvsp[(1) - (2)].m_pFunctionHeader), nullptr, nullptr, (yyvsp[(2) - (2)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 66:
/* Line 1792 of yacc.c  */
#line 355 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(2) - (4)].m_pIdentifier), nullptr, nullptr, false);
	}
    break;

  case 67:
/* Line 1792 of yacc.c  */
#line 358 "iris.y"
    {
		(yyval.m_pFunctionHeader) = new IrisFunctionHeader((yyvsp[(2) - (5)].m_pIdentifier), (yyvsp[(4) - (5)].m_pIdentifierList), nullptr, false);
	}
    break;

  case 68:
/* Line 1792 of yacc.c  */
#line 369 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "[]");
	}
    break;

  case 69:
/* Line 1792 of yacc.c  */
#line 372 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "[]=");
	}
    break;

  case 70:
/* Line 1792 of yacc.c  */
#line 375 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "+");
	}
    break;

  case 71:
/* Line 1792 of yacc.c  */
#line 378 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "||");
	}
    break;

  case 72:
/* Line 1792 of yacc.c  */
#line 381 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "!");
	}
    break;

  case 73:
/* Line 1792 of yacc.c  */
#line 384 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "==");
	}
    break;

  case 74:
/* Line 1792 of yacc.c  */
#line 387 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "!=");
	}
    break;

  case 75:
/* Line 1792 of yacc.c  */
#line 390 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, ">");
	}
    break;

  case 76:
/* Line 1792 of yacc.c  */
#line 393 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, ">=");
	}
    break;

  case 77:
/* Line 1792 of yacc.c  */
#line 396 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "<");
	}
    break;

  case 78:
/* Line 1792 of yacc.c  */
#line 399 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "<=");
	}
    break;

  case 79:
/* Line 1792 of yacc.c  */
#line 402 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "+");
	}
    break;

  case 80:
/* Line 1792 of yacc.c  */
#line 405 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "-");
	}
    break;

  case 81:
/* Line 1792 of yacc.c  */
#line 408 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "*");
	}
    break;

  case 82:
/* Line 1792 of yacc.c  */
#line 411 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "/");
	}
    break;

  case 83:
/* Line 1792 of yacc.c  */
#line 414 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "%");
	}
    break;

  case 84:
/* Line 1792 of yacc.c  */
#line 417 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "**");
	}
    break;

  case 85:
/* Line 1792 of yacc.c  */
#line 420 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "&");
	}
    break;

  case 86:
/* Line 1792 of yacc.c  */
#line 423 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "|");
	}
    break;

  case 87:
/* Line 1792 of yacc.c  */
#line 426 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "^");
	}
    break;

  case 88:
/* Line 1792 of yacc.c  */
#line 429 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "!");
	}
    break;

  case 89:
/* Line 1792 of yacc.c  */
#line 432 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, ">>");
	}
    break;

  case 90:
/* Line 1792 of yacc.c  */
#line 435 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "<<");
	}
    break;

  case 91:
/* Line 1792 of yacc.c  */
#line 438 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, ">>>");
	}
    break;

  case 92:
/* Line 1792 of yacc.c  */
#line 441 "iris.y"
    {
		(yyval.m_pIdentifier) = new IrisIdentifier(IrisIdentifilerType::GlobalVariable, "<<<");
	}
    break;

  case 93:
/* Line 1792 of yacc.c  */
#line 448 "iris.y"
    {
		(yyvsp[(1) - (1)].m_pConditionIf)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = (yyvsp[(1) - (1)].m_pConditionIf);
	}
    break;

  case 94:
/* Line 1792 of yacc.c  */
#line 452 "iris.y"
    {
		(yyvsp[(1) - (1)].m_pLoopIf)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = (yyvsp[(1) - (1)].m_pLoopIf);
	}
    break;

  case 95:
/* Line 1792 of yacc.c  */
#line 459 "iris.y"
    {
		IrisConditionIfStatement* pConditionIf = new IrisConditionIfStatement((yyvsp[(3) - (5)].m_pExpression), (yyvsp[(5) - (5)].m_pBlock), nullptr, nullptr);
		(yyval.m_pConditionIf) = pConditionIf;
	}
    break;

  case 96:
/* Line 1792 of yacc.c  */
#line 463 "iris.y"
    {
		IrisConditionIfStatement* pConditionIf = new IrisConditionIfStatement((yyvsp[(3) - (7)].m_pExpression), (yyvsp[(5) - (7)].m_pBlock), nullptr, (yyvsp[(7) - (7)].m_pBlock));
		(yyval.m_pConditionIf) = pConditionIf;
	}
    break;

  case 97:
/* Line 1792 of yacc.c  */
#line 468 "iris.y"
    {
		IrisConditionIfStatement* pConditionIf = new IrisConditionIfStatement((yyvsp[(3) - (6)].m_pExpression), (yyvsp[(5) - (6)].m_pBlock), (yyvsp[(6) - (6)].m_pElseIfList), nullptr);
		(yyval.m_pConditionIf) = pConditionIf;
	}
    break;

  case 98:
/* Line 1792 of yacc.c  */
#line 472 "iris.y"
    {
		IrisConditionIfStatement* pConditionIf = new IrisConditionIfStatement((yyvsp[(3) - (8)].m_pExpression), (yyvsp[(5) - (8)].m_pBlock), (yyvsp[(6) - (8)].m_pElseIfList), (yyvsp[(8) - (8)].m_pBlock));
		(yyval.m_pConditionIf) = pConditionIf;
	}
    break;

  case 99:
/* Line 1792 of yacc.c  */
#line 479 "iris.y"
    {
		IrisList<IrisElseIf*>* pElseIfList = new IrisList<IrisElseIf*>();
		pElseIfList->Add((yyvsp[(1) - (1)].m_pElseIf));
		(yyval.m_pElseIfList) = pElseIfList;
	}
    break;

  case 100:
/* Line 1792 of yacc.c  */
#line 484 "iris.y"
    {
		(yyvsp[(1) - (2)].m_pElseIfList)->Add((yyvsp[(2) - (2)].m_pElseIf));
		(yyval.m_pElseIfList) = (yyvsp[(1) - (2)].m_pElseIfList);
	}
    break;

  case 101:
/* Line 1792 of yacc.c  */
#line 491 "iris.y"
    {
		IrisElseIf* pElseIf = new IrisElseIf((yyvsp[(3) - (5)].m_pExpression), (yyvsp[(5) - (5)].m_pBlock));
		(yyval.m_pElseIf) = pElseIf;
	}
    break;

  case 102:
/* Line 1792 of yacc.c  */
#line 498 "iris.y"
    {
		IrisLoopIfStatement* pLoopIf = new IrisLoopIfStatement((yyvsp[(3) - (8)].m_pExpression), (yyvsp[(5) - (8)].m_pExpression), (yyvsp[(6) - (8)].m_pIdentifier), (yyvsp[(8) - (8)].m_pBlock));
		(yyval.m_pLoopIf) = pLoopIf;
	}
    break;

  case 103:
/* Line 1792 of yacc.c  */
#line 505 "iris.y"
    {
		(yyval.m_pIdentifier) = nullptr;
	}
    break;

  case 104:
/* Line 1792 of yacc.c  */
#line 508 "iris.y"
    {
		(yyval.m_pIdentifier) = (yyvsp[(2) - (2)].m_pIdentifier);
	}
    break;

  case 105:
/* Line 1792 of yacc.c  */
#line 515 "iris.y"
    {
		IrisStatement* pStatement = new IrisSwitchStatement((yyvsp[(3) - (5)].m_pExpression), (yyvsp[(5) - (5)].m_pSwitchBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 106:
/* Line 1792 of yacc.c  */
#line 523 "iris.y"
    {
		IrisSwitchBlock* pSwitchBlock = new IrisSwitchBlock((yyvsp[(2) - (3)].m_pWhenList), nullptr);
		(yyval.m_pSwitchBlock) = pSwitchBlock;
	}
    break;

  case 107:
/* Line 1792 of yacc.c  */
#line 527 "iris.y"
    {
		IrisSwitchBlock* pSwitchBlock = new IrisSwitchBlock((yyvsp[(2) - (5)].m_pWhenList), (yyvsp[(4) - (5)].m_pBlock));
		(yyval.m_pSwitchBlock) = pSwitchBlock;		
	}
    break;

  case 108:
/* Line 1792 of yacc.c  */
#line 534 "iris.y"
    {
		IrisList<IrisWhen*>* pWhenList = new IrisList<IrisWhen*>();
		pWhenList->Add((yyvsp[(1) - (1)].m_pWhen));
		(yyval.m_pWhenList) = pWhenList;
	}
    break;

  case 109:
/* Line 1792 of yacc.c  */
#line 539 "iris.y"
    {
		(yyvsp[(1) - (2)].m_pWhenList)->Add((yyvsp[(2) - (2)].m_pWhen));
		(yyval.m_pWhenList) = (yyvsp[(1) - (2)].m_pWhenList);
	}
    break;

  case 110:
/* Line 1792 of yacc.c  */
#line 546 "iris.y"
    {
		IrisWhen* pWhen = new IrisWhen((yyvsp[(3) - (5)].m_pExpressionList), (yyvsp[(5) - (5)].m_pBlock));
		(yyval.m_pWhen) = pWhen;
	}
    break;

  case 111:
/* Line 1792 of yacc.c  */
#line 554 "iris.y"
    {
		IrisStatement* pStatement = new IrisForStatement((yyvsp[(3) - (7)].m_pIdentifier), nullptr, (yyvsp[(5) - (7)].m_pExpression), (yyvsp[(7) - (7)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 112:
/* Line 1792 of yacc.c  */
#line 559 "iris.y"
    {
		IrisStatement* pStatement = new IrisForStatement((yyvsp[(4) - (11)].m_pIdentifier), (yyvsp[(6) - (11)].m_pIdentifier), (yyvsp[(9) - (11)].m_pExpression), (yyvsp[(11) - (11)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 113:
/* Line 1792 of yacc.c  */
#line 568 "iris.y"
    {
		IrisStatement* pStatement = new IrisOrderStatement((yyvsp[(2) - (8)].m_pBlock), (yyvsp[(5) - (8)].m_pIdentifier), (yyvsp[(7) - (8)].m_pBlock), (yyvsp[(8) - (8)].m_pBlock));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 114:
/* Line 1792 of yacc.c  */
#line 576 "iris.y"
    {
		(yyval.m_pBlock) = nullptr;
	}
    break;

  case 115:
/* Line 1792 of yacc.c  */
#line 579 "iris.y"
    {
		(yyval.m_pBlock) = new IrisBlock((yyvsp[(3) - (4)].m_pStatementList));
	}
    break;

  case 116:
/* Line 1792 of yacc.c  */
#line 587 "iris.y"
    {
		IrisStatement* pStatement = new IrisReturnStatement(nullptr);
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 117:
/* Line 1792 of yacc.c  */
#line 592 "iris.y"
    {
		IrisStatement* pStatement = new IrisReturnStatement((yyvsp[(3) - (3)].m_pExpression));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 118:
/* Line 1792 of yacc.c  */
#line 601 "iris.y"
    {
		IrisStatement* pStatement = new IrisBreakStatement(nullptr);
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 119:
/* Line 1792 of yacc.c  */
#line 609 "iris.y"
    {
		IrisStatement* pStatement = new IrisContinueStatement(nullptr);
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 120:
/* Line 1792 of yacc.c  */
#line 618 "iris.y"
    {
		IrisStatement* pStatement = new IrisInterfaceFunctionStatement((yyvsp[(3) - (5)].m_pIdentifier), nullptr, nullptr);
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 121:
/* Line 1792 of yacc.c  */
#line 623 "iris.y"
    {
		IrisStatement* pStatement = new IrisInterfaceFunctionStatement((yyvsp[(3) - (6)].m_pIdentifier), (yyvsp[(5) - (6)].m_pIdentifierList), nullptr);
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 122:
/* Line 1792 of yacc.c  */
#line 628 "iris.y"
    {
		IrisStatement* pStatement = new IrisInterfaceFunctionStatement((yyvsp[(3) - (9)].m_pIdentifier), (yyvsp[(5) - (9)].m_pIdentifierList), (yyvsp[(8) - (9)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 123:
/* Line 1792 of yacc.c  */
#line 633 "iris.y"
    {
		IrisStatement* pStatement = new IrisInterfaceFunctionStatement((yyvsp[(3) - (7)].m_pIdentifier), nullptr, (yyvsp[(6) - (7)].m_pIdentifier));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 124:
/* Line 1792 of yacc.c  */
#line 642 "iris.y"
    {
		IrisStatement* pStatement = new IrisGroanStatement((yyvsp[(4) - (5)].m_pExpression));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 125:
/* Line 1792 of yacc.c  */
#line 651 "iris.y"
    {
		IrisStatement* pStatement = new IrisBlockStatement();
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 126:
/* Line 1792 of yacc.c  */
#line 660 "iris.y"
    {
		IrisStatement* pStatement = new IrisCastStatement(nullptr);
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 127:
/* Line 1792 of yacc.c  */
#line 665 "iris.y"
    {
		IrisStatement* pStatement = new IrisCastStatement((yyvsp[(4) - (5)].m_pExpressionList));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 128:
/* Line 1792 of yacc.c  */
#line 674 "iris.y"
    {
		IrisStatement* pStatement = new IrisSuperStatement(nullptr);
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;
	}
    break;

  case 129:
/* Line 1792 of yacc.c  */
#line 679 "iris.y"
    {
		IrisStatement* pStatement = new IrisSuperStatement((yyvsp[(4) - (5)].m_pExpressionList));
		pStatement->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
		(yyval.m_pStatement) = pStatement;	
	}
    break;

  case 130:
/* Line 1792 of yacc.c  */
#line 687 "iris.y"
    {
		IrisBlock* pBlock = new IrisBlock(nullptr);
		(yyval.m_pBlock) = pBlock;
	}
    break;

  case 131:
/* Line 1792 of yacc.c  */
#line 691 "iris.y"
    {
		IrisBlock* pBlock = new IrisBlock((yyvsp[(2) - (3)].m_pStatementList));
		(yyval.m_pBlock) = pBlock;		
	}
    break;

  case 132:
/* Line 1792 of yacc.c  */
#line 697 "iris.y"
    {
		IrisList<IrisStatement*>* pStatementList = new IrisList<IrisStatement*>();
		pStatementList->Add((yyvsp[(1) - (1)].m_pStatement));
		(yyval.m_pStatementList) = pStatementList;
	}
    break;

  case 133:
/* Line 1792 of yacc.c  */
#line 702 "iris.y"
    {
		(yyvsp[(1) - (2)].m_pStatementList)->Add((yyvsp[(2) - (2)].m_pStatement));
		(yyval.m_pStatementList) = (yyvsp[(1) - (2)].m_pStatementList);
	}
    break;

  case 135:
/* Line 1792 of yacc.c  */
#line 711 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression((yyvsp[(2) - (3)].m_eExpressionType), (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 136:
/* Line 1792 of yacc.c  */
#line 718 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::Assign;
	}
    break;

  case 137:
/* Line 1792 of yacc.c  */
#line 721 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignAdd;
	}
    break;

  case 138:
/* Line 1792 of yacc.c  */
#line 724 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignSub;
	}
    break;

  case 139:
/* Line 1792 of yacc.c  */
#line 727 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignMul;
	}
    break;

  case 140:
/* Line 1792 of yacc.c  */
#line 730 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignDiv;
	}
    break;

  case 141:
/* Line 1792 of yacc.c  */
#line 733 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignMod;
	}
    break;

  case 142:
/* Line 1792 of yacc.c  */
#line 736 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignBitAnd;
	}
    break;

  case 143:
/* Line 1792 of yacc.c  */
#line 739 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignBitOr;
	}
    break;

  case 144:
/* Line 1792 of yacc.c  */
#line 742 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignBitXor;
	}
    break;

  case 145:
/* Line 1792 of yacc.c  */
#line 745 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignBitShr;
	}
    break;

  case 146:
/* Line 1792 of yacc.c  */
#line 748 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignBitShl;
	}
    break;

  case 147:
/* Line 1792 of yacc.c  */
#line 751 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignBitSar;
	}
    break;

  case 148:
/* Line 1792 of yacc.c  */
#line 754 "iris.y"
    {
		(yyval.m_eExpressionType) = IrisBinaryExpressionType::AssignBitSal;
	}
    break;

  case 150:
/* Line 1792 of yacc.c  */
#line 761 "iris.y"
    {
		(yyval.m_pExpression) =  new IrisBinaryExpression(IrisBinaryExpressionType::LogicOr, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 152:
/* Line 1792 of yacc.c  */
#line 769 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::LogicAnd, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 154:
/* Line 1792 of yacc.c  */
#line 777 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::LogicBitOr, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 156:
/* Line 1792 of yacc.c  */
#line 785 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::LogicBitXor, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 158:
/* Line 1792 of yacc.c  */
#line 793 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::LogicBitAnd, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 160:
/* Line 1792 of yacc.c  */
#line 801 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::Equal, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 161:
/* Line 1792 of yacc.c  */
#line 805 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::NotEqual, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 163:
/* Line 1792 of yacc.c  */
#line 813 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::GreatThan, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 164:
/* Line 1792 of yacc.c  */
#line 817 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::GreatThanOrEqual, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 165:
/* Line 1792 of yacc.c  */
#line 821 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::LessThan, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 166:
/* Line 1792 of yacc.c  */
#line 825 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::LessThanOrEqual, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 168:
/* Line 1792 of yacc.c  */
#line 833 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::BitShr, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 169:
/* Line 1792 of yacc.c  */
#line 837 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::BitShl, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 170:
/* Line 1792 of yacc.c  */
#line 841 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::BitSar, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 171:
/* Line 1792 of yacc.c  */
#line 845 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::BitSal, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 173:
/* Line 1792 of yacc.c  */
#line 853 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::Add, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 174:
/* Line 1792 of yacc.c  */
#line 857 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::Sub, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 176:
/* Line 1792 of yacc.c  */
#line 865 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::Mul, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 177:
/* Line 1792 of yacc.c  */
#line 869 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::Div, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 178:
/* Line 1792 of yacc.c  */
#line 873 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::Mod, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 180:
/* Line 1792 of yacc.c  */
#line 881 "iris.y"
    {
		(yyval.m_pExpression) = new IrisBinaryExpression(IrisBinaryExpressionType::Power, (yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 182:
/* Line 1792 of yacc.c  */
#line 889 "iris.y"
    {
		(yyval.m_pExpression) = new IrisUnaryExpression(IrisUnaryExpressionType::LogicNot, (yyvsp[(2) - (2)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 183:
/* Line 1792 of yacc.c  */
#line 893 "iris.y"
    {
		(yyval.m_pExpression) = new IrisUnaryExpression(IrisUnaryExpressionType::BitNot, (yyvsp[(2) - (2)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 184:
/* Line 1792 of yacc.c  */
#line 897 "iris.y"
    {
		(yyval.m_pExpression) = new IrisUnaryExpression(IrisUnaryExpressionType::Minus, (yyvsp[(2) - (2)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 185:
/* Line 1792 of yacc.c  */
#line 901 "iris.y"
    {
		(yyval.m_pExpression) = new IrisUnaryExpression(IrisUnaryExpressionType::Plus, (yyvsp[(2) - (2)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 187:
/* Line 1792 of yacc.c  */
#line 909 "iris.y"
    {
		(yyval.m_pExpression) = new IrisIndexExpression((yyvsp[(1) - (4)].m_pExpression), (yyvsp[(3) - (4)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 188:
/* Line 1792 of yacc.c  */
#line 913 "iris.y"
    {
		(yyval.m_pExpression) = new IrisFunctionCallExpression((yyvsp[(1) - (6)].m_pExpression), (yyvsp[(3) - (6)].m_pIdentifier), nullptr, (yyvsp[(6) - (6)].m_pClosureBlockLiteral));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 189:
/* Line 1792 of yacc.c  */
#line 917 "iris.y"
    {
		(yyval.m_pExpression) = new IrisFunctionCallExpression((yyvsp[(1) - (7)].m_pExpression), (yyvsp[(3) - (7)].m_pIdentifier), (yyvsp[(5) - (7)].m_pExpressionList), (yyvsp[(7) - (7)].m_pClosureBlockLiteral));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 190:
/* Line 1792 of yacc.c  */
#line 921 "iris.y"
    {
		(yyval.m_pExpression) = new IrisFunctionCallExpression(nullptr, (yyvsp[(1) - (4)].m_pIdentifier), nullptr, (yyvsp[(4) - (4)].m_pClosureBlockLiteral));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 191:
/* Line 1792 of yacc.c  */
#line 925 "iris.y"
    {
		(yyval.m_pExpression) = new IrisFunctionCallExpression(nullptr, (yyvsp[(1) - (5)].m_pIdentifier), (yyvsp[(3) - (5)].m_pExpressionList), (yyvsp[(5) - (5)].m_pClosureBlockLiteral));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 192:
/* Line 1792 of yacc.c  */
#line 929 "iris.y"
    {
		(yyval.m_pExpression) = new IrisMemberExpression((yyvsp[(1) - (3)].m_pExpression), (yyvsp[(3) - (3)].m_pIdentifier));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 193:
/* Line 1792 of yacc.c  */
#line 936 "iris.y"
    {
		(yyval.m_pClosureBlockLiteral) = nullptr;
	}
    break;

  case 194:
/* Line 1792 of yacc.c  */
#line 939 "iris.y"
    {
		(yyval.m_pClosureBlockLiteral) = nullptr;
	}
    break;

  case 195:
/* Line 1792 of yacc.c  */
#line 942 "iris.y"
    {
		(yyval.m_pClosureBlockLiteral) = new IrisClosureBlockLiteral(nullptr, (yyvsp[(2) - (3)].m_pStatementList));
	}
    break;

  case 196:
/* Line 1792 of yacc.c  */
#line 945 "iris.y"
    {
		(yyval.m_pClosureBlockLiteral) = new IrisClosureBlockLiteral((yyvsp[(5) - (9)].m_pIdentifierList), (yyvsp[(8) - (9)].m_pStatementList));
	}
    break;

  case 198:
/* Line 1792 of yacc.c  */
#line 952 "iris.y"
    {
		(yyval.m_pExpression) = (yyvsp[(2) - (3)].m_pExpression);
	}
    break;

  case 199:
/* Line 1792 of yacc.c  */
#line 955 "iris.y"
    {
		(yyval.m_pExpression) = new IrisIdentifierExpression((yyvsp[(1) - (1)].m_pIdentifier));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 200:
/* Line 1792 of yacc.c  */
#line 959 "iris.y"
    {
		(yyval.m_pExpression) = (yyvsp[(1) - (1)].m_pExpression);
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 201:
/* Line 1792 of yacc.c  */
#line 963 "iris.y"
    {
		(yyval.m_pExpression) = (yyvsp[(1) - (1)].m_pExpression);
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 202:
/* Line 1792 of yacc.c  */
#line 967 "iris.y"
    {
		(yyval.m_pExpression) = (yyvsp[(1) - (1)].m_pExpression);
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 203:
/* Line 1792 of yacc.c  */
#line 971 "iris.y"
    {
		(yyval.m_pExpression) = new IrisInstantValueExpression(IrisInstantValueType::True);
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 204:
/* Line 1792 of yacc.c  */
#line 975 "iris.y"
    {
		(yyval.m_pExpression) = new IrisInstantValueExpression(IrisInstantValueType::False);
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 205:
/* Line 1792 of yacc.c  */
#line 979 "iris.y"
    {
		(yyval.m_pExpression) = new IrisInstantValueExpression(IrisInstantValueType::Nil);
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 206:
/* Line 1792 of yacc.c  */
#line 983 "iris.y"
    {
		(yyval.m_pExpression) = new IrisSelfExpression();
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 210:
/* Line 1792 of yacc.c  */
#line 993 "iris.y"
    {
		(yyval.m_pExpression) = new IrisFieldExpression(new IrisFieldIdentifier((yyvsp[(1) - (2)].m_pIdentifierList), (yyvsp[(2) - (2)].m_pIdentifier), false));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 211:
/* Line 1792 of yacc.c  */
#line 997 "iris.y"
    {
		(yyval.m_pExpression) = new IrisFieldExpression(new IrisFieldIdentifier((yyvsp[(2) - (3)].m_pIdentifierList), (yyvsp[(3) - (3)].m_pIdentifier), true));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 212:
/* Line 1792 of yacc.c  */
#line 1001 "iris.y"
    {
		(yyval.m_pExpression) = new IrisFieldExpression(new IrisFieldIdentifier(nullptr, (yyvsp[(2) - (2)].m_pIdentifier), true));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 213:
/* Line 1792 of yacc.c  */
#line 1008 "iris.y"
    {
		(yyval.m_pExpression) = new IrisArrayExpression(nullptr);
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 214:
/* Line 1792 of yacc.c  */
#line 1012 "iris.y"
    {
        (yyval.m_pExpression) = new IrisArrayExpression((yyvsp[(2) - (3)].m_pExpressionList));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
    }
    break;

  case 215:
/* Line 1792 of yacc.c  */
#line 1016 "iris.y"
    {
        (yyval.m_pExpression) = new IrisArrayExpression((yyvsp[(2) - (4)].m_pExpressionList));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
    }
    break;

  case 216:
/* Line 1792 of yacc.c  */
#line 1024 "iris.y"
    {
		IrisList<IrisIdentifier*>* pFieldIdentifier = new IrisList<IrisIdentifier*>();
		pFieldIdentifier->Add((yyvsp[(1) - (2)].m_pIdentifier));
		(yyval.m_pIdentifierList) = pFieldIdentifier;
	}
    break;

  case 217:
/* Line 1792 of yacc.c  */
#line 1029 "iris.y"
    {
		(yyvsp[(1) - (3)].m_pIdentifierList)->Add((yyvsp[(2) - (3)].m_pIdentifier));
		(yyval.m_pIdentifierList) = (yyvsp[(1) - (3)].m_pIdentifierList);
	}
    break;

  case 218:
/* Line 1792 of yacc.c  */
#line 1037 "iris.y"
    {
		IrisList<IrisIdentifier*>* pList = new IrisList<IrisIdentifier*>();
		pList->Add((yyvsp[(1) - (1)].m_pIdentifier));
		(yyval.m_pIdentifierList) = pList;
	}
    break;

  case 219:
/* Line 1792 of yacc.c  */
#line 1042 "iris.y"
    {
		(yyvsp[(1) - (3)].m_pIdentifierList)->Add((yyvsp[(3) - (3)].m_pIdentifier));
		(yyval.m_pIdentifierList) = (yyvsp[(1) - (3)].m_pIdentifierList);
	}
    break;

  case 220:
/* Line 1792 of yacc.c  */
#line 1049 "iris.y"
    {
		IrisList<IrisExpression*>* pExpressionList = new IrisList<IrisExpression*>();
		pExpressionList->Add((yyvsp[(1) - (1)].m_pExpression));
		(yyval.m_pExpressionList) = pExpressionList;
	}
    break;

  case 221:
/* Line 1792 of yacc.c  */
#line 1054 "iris.y"
    {
		(yyvsp[(1) - (3)].m_pExpressionList)->Add((yyvsp[(3) - (3)].m_pExpression));
		(yyval.m_pExpressionList) = (yyvsp[(1) - (3)].m_pExpressionList);
	}
    break;

  case 222:
/* Line 1792 of yacc.c  */
#line 1061 "iris.y"
    {
		(yyval.m_pExpression) = new IrisHashExpression(nullptr);
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 223:
/* Line 1792 of yacc.c  */
#line 1065 "iris.y"
    {
        (yyval.m_pExpression) = new IrisHashExpression((yyvsp[(2) - (3)].m_pHashPairList));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
    }
    break;

  case 224:
/* Line 1792 of yacc.c  */
#line 1069 "iris.y"
    {
		(yyvsp[(2) - (6)].m_pHashPairList)->Add(new IrisHashPair((yyvsp[(3) - (6)].m_pExpression), (yyvsp[(5) - (6)].m_pExpression)));
        (yyval.m_pExpression) = new IrisHashExpression((yyvsp[(2) - (6)].m_pHashPairList));
    }
    break;

  case 225:
/* Line 1792 of yacc.c  */
#line 1076 "iris.y"
    {
		IrisList<IrisHashPair*>* pHashPairList = new IrisList<IrisHashPair*>();
		pHashPairList->Add((yyvsp[(1) - (1)].m_pHashPair));
		(yyval.m_pHashPairList) = pHashPairList;
	}
    break;

  case 226:
/* Line 1792 of yacc.c  */
#line 1081 "iris.y"
    {
		(yyvsp[(1) - (2)].m_pHashPairList)->Add((yyvsp[(2) - (2)].m_pHashPair));
		(yyval.m_pHashPairList) = (yyvsp[(1) - (2)].m_pHashPairList);
	}
    break;

  case 227:
/* Line 1792 of yacc.c  */
#line 1087 "iris.y"
    {
		(yyval.m_pHashPair) = new IrisHashPair((yyvsp[(1) - (4)].m_pExpression), (yyvsp[(3) - (4)].m_pExpression));
	}
    break;

  case 228:
/* Line 1792 of yacc.c  */
#line 1092 "iris.y"
    {
		(yyval.m_pExpression) = new IrisRangeExpression(IrisRangeType::LeftOpenAndRightOpen, (yyvsp[(2) - (5)].m_pExpression), (yyvsp[(4) - (5)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());
	}
    break;

  case 229:
/* Line 1792 of yacc.c  */
#line 1096 "iris.y"
    {
		(yyval.m_pExpression) = new IrisRangeExpression(IrisRangeType::LeftOpenAndRightClosed, (yyvsp[(2) - (5)].m_pExpression), (yyvsp[(4) - (5)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());	
	}
    break;

  case 230:
/* Line 1792 of yacc.c  */
#line 1100 "iris.y"
    {
		(yyval.m_pExpression) = new IrisRangeExpression(IrisRangeType::LeftClosedAndRightClosed, (yyvsp[(2) - (5)].m_pExpression), (yyvsp[(4) - (5)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());		
	}
    break;

  case 231:
/* Line 1792 of yacc.c  */
#line 1104 "iris.y"
    {
		(yyval.m_pExpression) = new IrisRangeExpression(IrisRangeType::LeftClosedAndRightOpen, (yyvsp[(2) - (5)].m_pExpression), (yyvsp[(4) - (5)].m_pExpression));
		(yyval.m_pExpression)->SetLineNumber(IrisCompiler::CurrentCompiler()->GetCurrentLineNumber());			
	}
    break;


/* Line 1792 of yacc.c  */
#line 3775 "y.tab.c"
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
#line 1108 "iris.y"
