#ifndef _H_IRISMEMORYPOOLDEFINES_

#define INC_POOLID(upper) (upper + 1)

#define POOLID_IrisObject					(0)
#define POOLID_IrisModule					INC_POOLID(POOLID_IrisObject)
#define POOLID_IrisMethod					INC_POOLID(POOLID_IrisModule)
#define POOLID_IrisInterface				INC_POOLID(POOLID_IrisMethod)
#define POOLID_IrisContextEnvironment		INC_POOLID(POOLID_IrisInterface)
#define POOLID_IrisClosureBlock				INC_POOLID(POOLID_IrisContextEnvironment)
#define POOLID_IrisClass					INC_POOLID(POOLID_IrisClosureBlock)

#define POOLID_IrisArrayTag					INC_POOLID(POOLID_IrisClass)
#define POOLID_IrisClassBaseTag				INC_POOLID(POOLID_IrisArrayTag)
#define POOLID_IrisClosureBlockBaseTag		INC_POOLID(POOLID_IrisClassBaseTag)
#define POOLID_IrisFloatTag					INC_POOLID(POOLID_IrisClosureBlockBaseTag)
#define POOLID_IrisHashTag					INC_POOLID(POOLID_IrisFloatTag)
#define POOLID_IrisIntegerTag				INC_POOLID(POOLID_IrisHashTag)
#define POOLID_IrisInterfaceBaseTag			INC_POOLID(POOLID_IrisIntegerTag)
#define POOLID_IrisMethodBaseTag			INC_POOLID(POOLID_IrisInterfaceBaseTag)
#define POOLID_IrisModuleTag				INC_POOLID(POOLID_IrisMethodBaseTag)
#define POOLID_IrisModuleBaseTag			INC_POOLID(POOLID_IrisModuleTag)
#define POOLID_IrisRangeTag					INC_POOLID(POOLID_IrisModuleBaseTag)
#define POOLID_IrisStringTag				INC_POOLID(POOLID_IrisRangeTag)
#define POOLID_IrisUniqueStringTag			INC_POOLID(POOLID_IrisStringTag)

#define RESERVATION_COUNT					(POOLID_IrisUniqueStringTag + 1)

#endif
